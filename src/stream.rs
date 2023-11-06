use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// A [`Stream`] is a type that implements both AsyncRead and AsyncWrite representing making it a
/// generic full-duplex I/O stream.
pub trait Stream: AsyncRead + AsyncWrite + Unpin + Send + Sync {}
impl<T> Stream for T where T: AsyncRead + AsyncWrite + Unpin + Send + Sync {}

pub trait ReadHalf: AsyncRead + Unpin + Send + Sync {}
impl<T> ReadHalf for T where T: AsyncRead + Unpin + Send + Sync {}

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
