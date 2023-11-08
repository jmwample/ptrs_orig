#![allow(dead_code)]
use crate::socks5;
use ptrs::{Error, Result};
use tor_rtcompat::PreferredRuntime;

use async_compat::CompatExt;
use std::str::FromStr;

use tokio::{
    self,
    io::{copy, split, AsyncRead, AsyncWrite},
};
use tokio_util::sync::CancellationToken;
use tracing::trace;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Handler {
    Socks5,
    Echo(EchoHandler),
}

impl Handler {
    pub async fn handle<RW>(self, stream: RW, close_c: CancellationToken) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        match self {
            Handler::Socks5 => Socks5Handler::handle(stream.compat(), close_c).await,
            Handler::Echo(h) => h.handle(stream, close_c).await,
        }
    }
}

impl FromStr for Handler {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "socks5" => Ok(Handler::Socks5),
            "echo" => Ok(Handler::Echo(EchoHandler)),
            _ => Err(Error::Other("unknown handler".into())),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Socks5Handler;

impl Socks5Handler {
    pub async fn handle<RW>(stream: RW, close_c: CancellationToken) -> Result<()>
    where
        RW: futures::AsyncRead + futures::AsyncWrite + Unpin + Send + Sync + 'static,
    {
        let rt = PreferredRuntime::current()?;
        tokio::select! {
            _ = socks5::handle_socks_conn(rt, stream) => {
                trace!("echo finished")
            }
            _ = close_c.cancelled() => {}
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EchoHandler;

impl EchoHandler {
    async fn handle<'a, RW>(&self, stream: RW, close_c: CancellationToken) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + 'a,
    {
        let (mut reader, mut writer) = split(stream);
        tokio::select! {
            _ = copy(&mut reader, &mut writer) => {
                trace!("echo finished")
            }
            _ = close_c.cancelled() => {}
        }
        Ok(())
    }
}
