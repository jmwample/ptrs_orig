use crate::{stream::combine, Result, Stream, Transport};

use futures::Future;
use tokio::io::{AsyncRead, AsyncWrite};

pub trait Reveal {
    fn reveal<'a>(
        &self,
        r: Box<dyn AsyncRead + Unpin + Send + Sync + 'a>,
    ) -> Box<dyn AsyncRead + Unpin + Send + Sync + 'a>;
}

pub trait Seal {
    fn seal<'a>(
        &self,
        w: Box<dyn AsyncWrite + Unpin + Send + Sync + 'a>,
    ) -> Box<dyn AsyncWrite + Unpin + Send + Sync + 'a>;
}

pub trait WrapTransport {
    fn sealer(&self) -> Result<Wrapper>;

    fn revealer(&self) -> Result<Wrapper>;
}

pub struct Wrapper {
    pub seal: Box<dyn Seal + Unpin + Send + Sync>,
    pub reveal: Box<dyn Reveal + Unpin + Send + Sync>,
}

impl Wrapper {
    async fn wrap<'a, A>(&self, a: A) -> Result<Box<dyn Stream + 'a>>
    where
        A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
    {
        let (r1, w1) = tokio::io::split(a);
        let r_prime = self.reveal.reveal(Box::new(r1)); // seal outgoing stream
        let w_prime = self.seal.seal(Box::new(w1)); // reveal incoming stream
        Ok(Box::new(combine(r_prime, w_prime)))
    }
}

// #[async_trait]
impl<'a, A> Transport<'a, A> for Wrapper
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn wrap(&self, a: A) -> impl Future<Output = Result<Box<dyn Stream + 'a>>> {
        self.wrap(a)
    }
}

#[cfg(test)]
mod test {
    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

    fn wrap_read<R: AsyncRead + Unpin>(r: R) -> impl AsyncRead {
        r
    }

    #[tokio::test]
    async fn test_wrap_read() {
        let (mut client, mut server) = tokio::net::UnixStream::pair().unwrap();
        tokio::spawn(async move {
            let (r, mut w) = server.split();

            let mut wrapped_r = wrap_read(r);

            tokio::io::copy(&mut wrapped_r, &mut w).await.unwrap();
        });

        let nw = client.write(&[0_u8; 1024]).await.unwrap();
        assert_eq!(nw, 1024);

        let mut buf = [0_u8; 1024];
        let nr = client.read(&mut buf).await.unwrap();
        assert_eq!(nr, 1024);
    }
}
