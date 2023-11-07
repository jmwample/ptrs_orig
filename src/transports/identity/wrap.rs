use super::Identity;
use crate::pt::wrap::*;
use crate::Result;

use tokio::io::{AsyncRead, AsyncWrite};

impl Seal for Identity {
    fn seal<'a>(
        &self,
        w: Box<dyn AsyncWrite + Unpin + Send + Sync + 'a>,
    ) -> Box<dyn AsyncWrite + Unpin + Send + Sync + 'a> {
        w
    }
}

impl Reveal for Identity {
    fn reveal<'a>(
        &self,
        r: Box<dyn AsyncRead + Unpin + Send + Sync + 'a>,
    ) -> Box<dyn AsyncRead + Unpin + Send + Sync + 'a> {
        r
    }
}

impl WrapTransport for Identity {
    fn wrapper(
        &self,
    ) -> Result<(
        Box<dyn Seal + Unpin + Send + Sync>,
        Box<dyn Reveal + Unpin + Send + Sync>,
    )> {
        Ok((Box::new(*self), Box::new(*self)))
    }

    fn unwrapper(
        &self,
    ) -> Result<(
        Box<dyn Seal + Unpin + Send + Sync>,
        Box<dyn Reveal + Unpin + Send + Sync>,
    )> {
        Ok((Box::new(*self), Box::new(*self)))
    }
}
