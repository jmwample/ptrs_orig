#![allow(dead_code)]
use ptrs::{Error, Result};

use std::str::FromStr;

use super::socks5::handle;
use socks5_server::{auth::NoAuth, Server};
use tokio::{
    self,
    io::{copy, AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, trace};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Handler {
    // Socks5(Socks5Handler),
    Echo(EchoHandler),
}

impl Handler {
    pub async fn handle_listener(
        &self,
        listener: TcpListener,
        close_c: CancellationToken,
    ) -> Result<()> {
        match self {
            // Handler::Socks5(h) => h.handle_listener(listener, close_c).await,
            Handler::Echo(h) => h.handle_listener(listener, close_c).await,
        }
    }

    pub async fn handle(self, stream: TcpStream, close_c: CancellationToken) -> Result<()>
// where
        // RW: Split + AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        match self {
            // Handler::Socks5(h) => h.handle(stream, close_c).await,
            Handler::Echo(h) => h.handle(stream, close_c).await,
        }
    }
}

impl FromStr for Handler {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            // "socks5" => Ok(Handler::Socks5(Socks5Handler)),
            "echo" => Ok(Handler::Echo(EchoHandler)),
            _ => Err(Error::Other("unknown handler".into())),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Socks5Handler;

// impl Socks5Handler {
//     async fn handle_listener(
//         &self,
//         listener: TcpListener,
//         close_c: CancellationToken,
//     ) -> Result<()> {
//         let auth = Arc::new(NoAuth) as Arc<_>;
//         let server = Server::new(listener, auth);
//         'outer: loop {
//             tokio::select!(
//                 res = server.accept() => {
//                     let (stream, socket_addr) = res?;
//                     debug!("new connection {socket_addr}");
//                     let close = close_c.clone();
//                     tokio::spawn( async move {
//                         tokio::select! {
//                             res = handle(stream) => {
//                                 match res {
//                                     Ok(()) => {}
//                                     Err(err) => eprintln!("stream error {socket_addr}: {err}"),
//                                 }
//                             }
//                             _ = close.cancelled() => {
//                                 trace!("closing {socket_addr}");
//                             }
//                         }
//                     });
//                 }
//                 _ = close_c.cancelled() => {
//                     break 'outer;
//                 }
//             )
//         }
//         debug!("shutting down server listen handler");
//         Ok(())
//     }

//     pub async fn handle<RW>(&self, _stream: RW, _close_c: CancellationToken) -> Result<()>
//     where
//         RW: Split + AsyncRead + AsyncWrite + Unpin + Send + 'static,
//     {
//         Err(Error::Other("not implemented".into()))
//         // Not sure how to do this for now. Socks5 server implementations are few and far between.
//         // maybe the tor socks implementation, but that seems more involved.
//     }
// }

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EchoHandler;

impl EchoHandler {
    async fn handle_listener(
        &self,
        listener: TcpListener,
        close_c: CancellationToken,
    ) -> Result<()> {
        'outer: loop {
            tokio::select!(
                res = listener.accept() => {
                    let (stream, socket_addr) = res?;
                    debug!("new connection {socket_addr}");
                    let close = close_c.clone();
                    tokio::spawn( async move {
                        let (mut reader, mut writer) = tokio::io::split(stream);
                        tokio::select! {
                            _ = copy(&mut reader, &mut writer) => {}
                            _ = close.cancelled() => {}
                        }
                    });
                }
                _ = close_c.cancelled() => {
                    break 'outer;
                }
            )
        }
        Ok(())
    }

    async fn handle(&self, mut stream: TcpStream, close_c: CancellationToken) -> Result<()>
// where
    //     RW: Split<'a> + AsyncRead + AsyncWrite + Unpin + Send + 'a,
    {
        let (mut reader, mut writer) = stream.split();
        tokio::select! {
            _ = copy(&mut reader, &mut writer) => {}
            _ = close_c.cancelled() => {}
        }
        Ok(())
    }
}
