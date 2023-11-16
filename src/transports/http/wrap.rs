use crate::{pt::wrap::*, Result};
use tokio::io::{AsyncRead, AsyncWrite};

use super::Http;

impl Seal for Http {
    fn seal<'a>(
        &self,
        w: Box<dyn AsyncWrite + Unpin + Send + Sync + 'a>,
    ) -> Box<dyn AsyncWrite + Unpin + Send + Sync + 'a> {
        w
    }
}

impl Reveal for Http {
    fn reveal<'a>(
        &self,
        r: Box<dyn AsyncRead + Unpin + Send + Sync + 'a>,
    ) -> Box<dyn AsyncRead + Unpin + Send + Sync + 'a> {
        r
    }
}

impl WrapTransport for Http {
    fn sealer(&self) -> Result<Wrapper> {
        Ok(Wrapper {
            seal: Box::new(*self),
            reveal: Box::new(*self),
            name: "http",
        })
    }

    fn revealer(&self) -> Result<Wrapper> {
        Ok(Wrapper {
            seal: Box::new(*self),
            reveal: Box::new(*self),
            name: "http",
        })
    }
}
