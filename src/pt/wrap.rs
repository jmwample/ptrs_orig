use crate::Result;

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
    fn wrapper(
        &self,
    ) -> Result<(
        Box<dyn Seal + Unpin + Send + Sync>,
        Box<dyn Reveal + Unpin + Send + Sync>,
    )>;

    fn unwrapper(
        &self,
    ) -> Result<(
        Box<dyn Seal + Unpin + Send + Sync>,
        Box<dyn Reveal + Unpin + Send + Sync>,
    )>;
}

#[cfg(test)]
mod test {
    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

    fn wrap_read<R: AsyncRead + Unpin>(r: R) -> impl AsyncRead {
        return r;
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
