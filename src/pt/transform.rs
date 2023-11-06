use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub trait BufferTransform {
    fn poll_copy<R, W>(
        &mut self,
        cx: &mut Context<'_>,
        reader: Pin<&mut R>,
        writer: Pin<&mut W>,
    ) -> Poll<io::Result<u64>>
    where
        R: AsyncRead + ?Sized,
        W: AsyncWrite + ?Sized;
}

#[pin_project]
pub struct ReadTransform<T, R>
where
    R: AsyncRead + Unpin + Send + Sync,
    T: BufferTransform + Unpin + Send + Sync,
{
    inner: T,
    #[pin]
    r: R,
}

impl<'a, T, R> ReadTransform<T, R>
where
    R: AsyncRead + Unpin + Send + Sync + 'a,
    T: BufferTransform + Unpin + Send + Sync + 'a,
{
    pub fn new(r: R, inner: T) -> Box<dyn AsyncRead + Unpin + Send + Sync + 'a> {
        Box::new(Self { inner, r })
    }
}

#[pin_project]
pub struct WriteTransform<T, W>
where
    W: AsyncWrite + Unpin + Send + Sync,
    T: BufferTransform + Unpin + Send + Sync,
{
    inner: T,
    #[pin]
    w: W,
}

impl<'a, T, W> WriteTransform<T, W>
where
    W: AsyncWrite + Unpin + Send + Sync + 'a,
    T: BufferTransform + Unpin + Send + Sync + 'a,
{
    pub fn new(w: W, inner: T) -> Box<dyn AsyncWrite + Unpin + Send + Sync + 'a>
    where
        T: BufferTransform + Unpin + Send + Sync + 'a,
        W: AsyncWrite + Unpin + Send + Sync + 'a,
    {
        Box::new(Self { inner, w })
    }
}

impl<T, R> AsyncRead for ReadTransform<T, R>
where
    R: AsyncRead + Unpin + Send + Sync,
    T: BufferTransform + Unpin + Send + Sync,
{
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut ReadBuf,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.as_mut().project();
        this.r.poll_read(cx, buf)
    }
}

impl<T, W> AsyncWrite for WriteTransform<T, W>
where
    W: AsyncWrite + Unpin + Send + Sync,
    T: BufferTransform + Unpin + Send + Sync,
{
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Error, Result};

    #[test]
    fn test_placeholder() -> Result<()> {
        Err(Error::Other("not implemented yet".into()))
    }
}
