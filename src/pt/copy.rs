use crate::pt::copy_buffer::*;
use crate::{Error, Result};

use futures::{future::poll_fn, ready};
use tokio::io::{AsyncRead, AsyncWrite};

use async_trait::async_trait;

use std::io;
use std::task::{Context, Poll};

pub enum TransferState {
    Running(CopyBuffer),
    ShuttingDown(u64),
    Done(u64),
}

pub trait SimplexTransform<A: ?Sized, B: ?Sized>: Send + Sync {
    fn transfer_one_direction(
        &self,
        cx: &mut Context<'_>,
        state: &mut TransferState,
        r: &mut A,
        w: &mut B,
    ) -> Poll<io::Result<u64>>;
}

impl<A, B, S> SimplexTransform<A, B> for Box<S>
where
    S: SimplexTransform<A, B> + ?Sized,
{
    fn transfer_one_direction(
        &self,
        cx: &mut Context<'_>,
        state: &mut TransferState,
        r: &mut A,
        w: &mut B,
    ) -> Poll<io::Result<u64>> {
        (**self).transfer_one_direction(cx, state, r, w)
    }
}

impl<A, B, S: SimplexTransform<A, B> + ?Sized> SimplexTransform<A, B> for &'_ S {
    fn transfer_one_direction(
        &self,
        cx: &mut Context<'_>,
        state: &mut TransferState,
        r: &mut A,
        w: &mut B,
    ) -> Poll<io::Result<u64>> {
        (**self).transfer_one_direction(cx, state, r, w)
    }
}

#[async_trait]
pub trait DuplexTransform<A, B>
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    async fn copy_bidirectional<'a, 'b>(
        &self,
        a: &'a mut A,
        b: &'b mut B,
    ) -> std::result::Result<(u64, u64), std::io::Error>
    where
        A: AsyncRead + AsyncWrite + Unpin,
        B: AsyncRead + AsyncWrite + Unpin;
}

pub fn duplex_from_simplices<'t, 's, A, B, T1, T2>(t1: T1, t2: T2) -> DuplexFromSimplices<'t, A, B>
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 's,
    B: AsyncRead + AsyncWrite + Unpin + Send + Sync + 's,
    T1: SimplexTransform<A, B> + 't,
    T2: SimplexTransform<B, A> + 't,
    't: 's,
{
    DuplexFromSimplices {
        t1: Box::new(t1),
        t2: Box::new(t2),
    }
}

pub struct DuplexFromSimplices<'t, A, B>
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    t1: Box<dyn SimplexTransform<A, B> + 't>,
    t2: Box<dyn SimplexTransform<B, A> + 't>,
}

#[async_trait]
impl<'t, A, B> DuplexTransform<A, B> for DuplexFromSimplices<'t, A, B>
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync,
    B: AsyncRead + AsyncWrite + Unpin + Send + Sync,
{
    async fn copy_bidirectional<'a, 'b>(
        &self,
        a: &'a mut A,
        b: &'b mut B,
    ) -> std::result::Result<(u64, u64), std::io::Error> {
        let mut a_to_b = TransferState::Running(CopyBuffer::new());
        let mut b_to_a = TransferState::Running(CopyBuffer::new());
        poll_fn(move |cx| {
            let a_to_b = self.t1.transfer_one_direction(cx, &mut a_to_b, a, b)?;
            let b_to_a = self.t2.transfer_one_direction(cx, &mut b_to_a, b, a)?;

            // It is not a problem if ready! returns early because transfer_one_direction for the
            // other direction will keep returning TransferState::Done(count) in future calls to poll
            let a_to_b = ready!(a_to_b);
            let b_to_a = ready!(b_to_a);

            Poll::Ready(Ok((a_to_b, b_to_a)))
        })
        .await
    }
}

pub(crate) fn duplex_from_transform_buffer<T, A, B>(
    _transform: T,
) -> Result<Box<dyn DuplexTransform<A, B>>>
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    Err(Error::Other("Not implemented yet".into()))
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::{join, try_join};
    use std::pin::Pin;
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::unix::WriteHalf;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn copy_test() {
        let (mut client, mut server) = tokio::net::UnixStream::pair().unwrap();
        let server_task = tokio::spawn(async move {
            let mut buf = [0_u8; 1024];
            let nr = server.read(&mut buf).await.unwrap();
            assert_eq!(nr, 1024);
            let nw = server.write(&buf[..nr]).await.unwrap();
            assert_eq!(nw, 1024);
        });

        let client_task = tokio::spawn(async move {
            let mut buf = [0_u8; 1024];
            let nw = client.write(&buf).await.unwrap();
            assert_eq!(nw, 1024);
            let nr = client.read(&mut buf).await.unwrap();
            assert_eq!(nr, 1024);
        });

        try_join!(client_task, server_task).unwrap();
    }

    ///
    ///						 write 	 ===================>    encode   ===================>  >|
    ///						 read 	 <===================    decode   <===================  <| echo
    ///
    ///        [ loop Buffer ] -> | source | -> | plaintext | -> | ciphertext | -> | echo |
    ///									    pipe						        pipe
    ///
    #[allow(non_snake_case)]
    #[tokio::test]
    async fn stream_transform_end_to_end_1_MB() {
        let (mut source, mut plaintext) = tokio::net::UnixStream::pair().unwrap();
        let (mut ciphertext, mut echo) = tokio::net::UnixStream::pair().unwrap();

        let out_file = tokio::fs::File::create("/dev/null").await.unwrap();
        let mut out_file = tokio::io::BufWriter::new(out_file);

        let transport = TestStream {};

        let proxy_task = transport.copy_bidirectional(&mut plaintext, &mut ciphertext);

        let echo_task = tokio::spawn(async move {
            let (mut echo_r, mut echo_w) = echo.split();
            let total = tokio::io::copy(&mut echo_r, &mut echo_w).await.unwrap();
            assert_eq!(total, 1024 * 1024);
        });

        let trash_task = tokio::spawn(async move {
            let (mut source_r, source_w) = source.split();
            let trash_copy = tokio::io::copy(&mut source_r, &mut out_file);

            let a_source_w = Arc::new(Mutex::new(source_w));
            let client_write = write_and_close(a_source_w);

            let (trash_total, write_total) = try_join!(trash_copy, client_write,).unwrap();
            assert_eq!(trash_total, 1024 * 1024);
            assert_eq!(write_total, 1024 * 1024);
        });

        let (r1, r2, r3) = join!(trash_task, proxy_task, echo_task,);
        r1.unwrap();
        r2.unwrap();
        r3.unwrap();
    }

    async fn write_and_close(w: Arc<Mutex<WriteHalf<'_>>>) -> std::io::Result<usize> {
        let write_me = vec![0_u8; 1024];
        let mut locked_w = w.lock().await;
        let mut n = 0;
        for _ in 0..1023 {
            n += locked_w.write(&write_me).await?;
        }
        n += locked_w.write(&write_me).await?;
        std::thread::sleep(std::time::Duration::from_millis(100));
        locked_w.shutdown().await?;
        Ok(n)
    }

    struct TestStream {}

    impl TestStream {
        pub async fn copy_bidirectional<A, B>(&self, a: &mut A, b: &mut B) -> Result<(u64, u64)>
        where
            A: AsyncRead + AsyncWrite + Unpin + ?Sized,
            B: AsyncRead + AsyncWrite + Unpin + ?Sized,
        {
            let mut a_to_b = TransferState::Running(CopyBuffer::new());
            let mut b_to_a = TransferState::Running(CopyBuffer::new());
            poll_fn(|cx| {
                let a_to_b = transfer_one_direction(cx, &mut a_to_b, a, b)?;
                let b_to_a = transfer_one_direction(cx, &mut b_to_a, b, a)?;

                // It is not a problem if ready! returns early because transfer_one_direction for the
                // other direction will keep returning TransferState::Done(count) in future calls to poll
                let a_to_b = ready!(a_to_b);
                let b_to_a = ready!(b_to_a);

                Poll::Ready(Ok((a_to_b, b_to_a)))
            })
            .await
        }
    }

    fn transfer_one_direction<A, B>(
        cx: &mut Context<'_>,
        state: &mut TransferState,
        r: &mut A,
        w: &mut B,
    ) -> Poll<io::Result<u64>>
    where
        A: AsyncRead + AsyncWrite + Unpin + ?Sized,
        B: AsyncRead + AsyncWrite + Unpin + ?Sized,
    {
        let mut r = Pin::new(r);
        let mut w = Pin::new(w);

        loop {
            match state {
                TransferState::Running(buf) => {
                    let count = ready!(buf.poll_copy(cx, r.as_mut(), w.as_mut()))?;
                    *state = TransferState::ShuttingDown(count);
                }
                TransferState::ShuttingDown(count) => {
                    ready!(w.as_mut().poll_shutdown(cx))?;

                    *state = TransferState::Done(*count);
                }
                TransferState::Done(count) => return Poll::Ready(Ok(*count)),
            }
        }
    }
}
