use tokio::io::{AsyncRead, AsyncWrite};

use crate::{pt::stream::StreamTransport, stream::Stream};

use super::*;

impl<'a, A> StreamTransport<'a, A> for Identity
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn wrap(&self, a: A) -> Result<Box<dyn Stream + 'a>> {
        Ok(Box::new(a))
    }
}
