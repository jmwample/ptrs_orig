use std::io;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub trait BufferTransform<'a, R, W>
where
    R: AsyncRead + ?Sized + 'a,
    W: AsyncWrite + ?Sized + 'a,
{
    fn poll_copy(
        &mut self,
        cx: &mut Context<'_>,
        reader: Pin<&mut R>,
        writer: Pin<&mut W>,
    ) -> Poll<io::Result<u64>>;
}

impl<'a, R, W> BufferTransform<'a, R, W> for Box<dyn BufferTransform<'a, R, W> + 'a>
where
    R: AsyncRead + Unpin + Send + Sync + ?Sized + 'a,
    W: AsyncWrite + Unpin + Send + Sync + ?Sized + 'a,
{
    fn poll_copy(
        &mut self,
        cx: &mut Context<'_>,
        reader: Pin<&mut R>,
        writer: Pin<&mut W>,
    ) -> Poll<io::Result<u64>> {
        (**self).poll_copy(cx, reader, writer)
    }
}

// impl<'a, R, W> BufferTransform<'a, Box<R>, Box<W>> for Box<dyn BufferTransform<'a,R,W> + 'a>
// where
//     R: AsyncRead + Unpin + Send + Sync+ ?Sized + 'a,
//     W: AsyncWrite+ Unpin + Send + Sync + ?Sized + 'a,
//     Box<R>: AsyncRead + Unpin + Send + Sync + 'a,
//     Box<W>: AsyncWrite+ Unpin + Send + Sync + 'a,
// {
//     fn poll_copy(
//         &mut self,
//         cx: &mut Context<'_>,
//         reader: Pin<&mut Box<R>>,
//         writer: Pin<&mut Box<W>>,
//     ) -> Poll<io::Result<u64>> {
//         self.as_ref().poll_copy(cx, reader, writer)
//     }
// }

// impl<'a, R, W> BufferTransform<'a, &'a mut R, &'a mut W> for Box<dyn BufferTransform<'a,R,W> + 'a>
// where
//     R: AsyncRead + Unpin + Send + Sync+ ?Sized + 'a,
//     W: AsyncWrite+ Unpin + Send + Sync + ?Sized + 'a,
// {
//     fn poll_copy(
//         &mut self,
//         cx: &mut Context<'_>,
//         reader: Pin<&mut &mut R>,
//         writer: Pin<&mut &mut W>,
//     ) -> Poll<io::Result<u64>> {
//         self.as_ref().poll_copy(cx, reader, writer)
//     }
// }

#[pin_project]
pub struct ReadTransform<'a, T, R, W>
where
    R: AsyncRead + Unpin + Send + Sync + 'a,
    W: AsyncWrite + Unpin + Send + Sync + 'a,
    T: BufferTransform<'a, R, W> + Unpin + Send + Sync + 'a,
{
    inner: T,
    #[pin]
    r: R,
    _phantom: PhantomData<&'a W>,
}

impl<'a, T, R, W> ReadTransform<'a, T, R, W>
where
    R: AsyncRead + Unpin + Send + Sync + 'a,
    W: AsyncWrite + Unpin + Send + Sync + 'a,
    T: BufferTransform<'a, R, W> + Unpin + Send + Sync + 'a,
{
    pub fn new(r: R, inner: T) -> Self {
        Self {
            inner,
            r,
            _phantom: PhantomData,
        }
    }

    pub fn as_reader(self) -> Box<dyn AsyncRead + Unpin + Send + Sync + 'a> {
        Box::new(self)
    }
}

#[pin_project]
pub struct WriteTransform<'a, T, R, W>
where
    R: AsyncRead + Unpin + Send + Sync + 'a,
    W: AsyncWrite + Unpin + Send + Sync + 'a,
    T: BufferTransform<'a, R, W> + Unpin + Send + Sync + 'a,
{
    inner: T,
    #[pin]
    w: W,
    _phantom: PhantomData<&'a R>,
}

impl<'a, T, R, W> WriteTransform<'a, T, R, W>
where
    R: AsyncRead + Unpin + Send + Sync + 'a,
    W: AsyncWrite + Unpin + Send + Sync + 'a,
    T: BufferTransform<'a, R, W> + Unpin + Send + Sync + 'a,
{
    pub fn new(w: W, inner: T) -> Self {
        Self {
            inner,
            w,
            _phantom: PhantomData,
        }
    }
    pub fn as_writer(self) -> Box<dyn AsyncWrite + Unpin + Send + Sync + 'a> {
        Box::new(self)
    }
}

impl<'a, T, R, W> AsyncRead for ReadTransform<'a, T, R, W>
where
    R: AsyncRead + Unpin + Send + Sync + 'a,
    W: AsyncWrite + Unpin + Send + Sync + 'a,
    T: BufferTransform<'a, R, W> + Unpin + Send + Sync + 'a,
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

impl<'a, T, R, W> AsyncWrite for WriteTransform<'a, T, R, W>
where
    R: AsyncRead + Unpin + Send + Sync,
    W: AsyncWrite + Unpin + Send + Sync,
    T: BufferTransform<'a, R, W> + Unpin + Send + Sync,
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use futures::executor::block_on;
//     use tokio::io::AsyncReadExt;

//     struct ExampleTransform {}

//     impl<'a,R,W> BufferTransform<'a,R,W> for ExampleTransform
//     where
//         R: AsyncRead + Unpin + Send + Sync + ?Sized + 'a,
//         W: AsyncWrite + Unpin + Send + Sync + ?Sized + 'a,
//     {
//         fn poll_copy(
//             &mut self,
//             cx: &mut Context<'_>,
//             reader: Pin<&mut R>,
//             writer: Pin<&mut W>,
//         ) -> Poll<io::Result<u64>> {
//             let b = [0; 1024];
//             let mut buf = ReadBuf::new(&mut b);
//             let mut total = 0;
//             loop {
//                 let n = match reader.poll_read(cx, &mut buf) {
//                     Poll::Ready(Ok(_)) => buf.filled().len(),
//                     Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
//                     Poll::Pending => return Poll::Ready(Ok(total)),
//                 };
//                 if n == 0 {
//                     return Poll::Ready(Ok(total));
//                 }
//                 total += n as u64;
//                 match writer.poll_write(cx, &b[..n]) {
//                     Poll::Ready(Ok(_)) => {}
//                     Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
//                     Poll::Pending => return Poll::Ready(Ok(total)),
//                 }
//             }
//         }
//     }

//     #[test]
//     fn test_read_transform() {
//         // Create a buffer transform that just copies data from the input to the output.
//         let transform = ExampleTransform {};

//         // Create a reader that reads from a cursor.
//         let input = &b"hello world";
//         let (mut client, mut server) = tokio::io::duplex(64);
//         let reader = ReadTransform::new(client, transform);

//         // Read the data from the reader and check that it matches the input.
//         let mut buf = [0; 11];
//         block_on(reader.read_exact(&mut buf)).unwrap();
//         assert_eq!(&buf, b"hello world");
//     }
// }
