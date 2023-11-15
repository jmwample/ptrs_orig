use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::Result;

/// Abstraction over I/O interfaces requiring only that they implements
/// [AsyncRead], [AsyncWrite], and are safe to send between threads.
pub trait Stream: AsyncRead + AsyncWrite + Unpin + Send + Sync {}
impl<T> Stream for T where T: AsyncRead + AsyncWrite + Unpin + Send + Sync {}

/// Abstraction over I/O interfaces requiring only that they implements
/// [AsyncRead] and are safe to send between threads. Generally used in context
/// with a `split_*` or [`combine`] operation.
pub trait ReadHalf: AsyncRead + Unpin + Send + Sync {}
impl<T> ReadHalf for T where T: AsyncRead + Unpin + Send + Sync {}

/// Abstraction over I/O interfaces requiring only that they implements
/// [AsyncWrite] and are safe to send between threads. Generally used in context
/// with a `split_*` or [`combine`] operation.
pub trait WriteHalf: AsyncWrite + Unpin + Send + Sync {}
impl<T> WriteHalf for T where T: AsyncWrite + Unpin + Send + Sync {}

#[pin_project]
struct Combined<R, W> {
    #[pin]
    r: R,
    #[pin]
    w: W,
}

pub fn combine<R, W>(r: R, w: W) -> impl Stream
where
    R: AsyncRead + Unpin + Send + Sync,
    W: AsyncWrite + Unpin + Send + Sync,
{
    Combined { r, w }
}

impl<R: AsyncRead, W> AsyncRead for Combined<R, W> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut ReadBuf,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.project();
        this.r.poll_read(cx, buf)
    }
}

impl<R, W: AsyncWrite> AsyncWrite for Combined<R, W> {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let this = self.as_mut().project();
        this.w.poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.as_mut().project();
        this.w.poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.as_mut().project();
        this.w.poll_shutdown(cx)
    }
}



pub fn split_stream<'s, S>(
    s: S,
) -> Result<(
    Box<dyn ReadHalf + 's>,
    Box<dyn WriteHalf + 's>,
)>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + Sync + 's,
{
    let (r, w) = tokio::io::split(s);
    Ok((Box::new(r), Box::new(w)))
}

pub fn split_impl<'s, S>(
    s: S,
) -> Result<(
    impl AsyncRead + Unpin + Send + Sync + 's,
    impl AsyncWrite + Unpin + Send + Sync + 's,
)>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + Sync + 's,
{
    let (r, w) = tokio::io::split(s);
    Ok((r, w))
}

pub fn split_box<'s, S>(
    s: S,
) -> Result<(
    Box<dyn AsyncRead + Unpin + Send + Sync + 's>,
    Box<dyn AsyncWrite + Unpin + Send + Sync + 's>,
)>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + Sync + 's,
{
    let (r, w) = tokio::io::split(s);
    Ok((Box::new(r), Box::new(w)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixStream;

    #[tokio::test]
    async fn splits() -> Result<()> {
        let (client, server) = UnixStream::pair()?;

        let (mut cr1, mut cw1) = split_stream(client)?;
        let (mut sr1, mut sw1) = split_stream(server)?;
        test_split_read_write(&mut cr1, &mut cw1, &mut sr1, &mut sw1).await?;

        let (client, server) = UnixStream::pair()?;
        let (mut cr2, mut cw2) = split_impl(client)?;
        let (mut sr2, mut sw2) = split_impl(server)?;
        test_split_read_write(&mut cr2, &mut cw2, &mut sr2, &mut sw2).await?;

        let (client, server) = UnixStream::pair()?;
        let (mut cr3, mut cw3) = split_box(client)?;
        let (mut sr3, mut sw3) = split_box(server)?;
        test_split_read_write(&mut cr3, &mut cw3, &mut sr3, &mut sw3).await?;
        Ok(())
    }

    async fn test_split_read_write<'a, R1, W1, R2, W2>(
        mut cr: R1,
        mut cw: W1,
        mut sr: R2,
        mut sw: W2,
    ) -> Result<()>
    where
        R1: AsyncRead + Unpin + Send + Sync + 'a,
        W1: AsyncWrite + Unpin + Send + Sync + 'a,
        R2: AsyncRead + Unpin + Send + Sync + 'a,
        W2: AsyncWrite + Unpin + Send + Sync + 'a,
    {
        let message = "hello world";

        cw.write_all(message.as_bytes()).await?;
        let mut buf = [0; 11];
        sr.read_exact(&mut buf).await?;
        assert_eq!(buf, message.as_bytes());

        let message = "goodbye";
        sw.write_all(message.as_bytes()).await?;
        let mut buf = [0; 7];
        cr.read_exact(&mut buf).await?;
        assert_eq!(buf, message.as_bytes());

        Ok(())
    }
}
