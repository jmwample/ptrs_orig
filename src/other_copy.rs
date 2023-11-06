#![allow(dead_code)]
use std::io::Result as IoResult;

use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Copy all the data from `reader` into `writer` until we encounter an EOF or
/// an error.
///
/// Unlike as futures::io::copy(), this function is meant for use with
/// interactive readers and writers, where the reader might pause for
/// a while, but where we want to send data on the writer as soon as
/// it is available.
///
/// This function assumes that the writer might need to be flushed for
/// any buffered data to be sent.  It tries to minimize the number of
/// flushes, however, by only flushing the writer when the reader has no data.
///
/// NOTE: This function is copied from the tor arti source code.
async fn copy_interactive<R, W>(mut reader: R, mut writer: W) -> IoResult<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    use futures::{poll, task::Poll};

    let mut buf = [0_u8; 1024];
    // At this point we could just loop, calling read().await,
    // write_all().await, and flush().await.  But we want to be more
    // clever than that: we only want to flush when the reader is
    // stalled.  That way we can pack our data into as few cells as
    // possible, but flush it immediately whenever there's no more
    // data coming.
    let loop_result: IoResult<()> = loop {
        let mut read_future = reader.read(&mut buf[..]);
        match poll!(&mut read_future) {
            Poll::Ready(Err(e)) => break Err(e),
            Poll::Ready(Ok(0)) => break Ok(()), // EOF
            Poll::Ready(Ok(n)) => {
                writer.write_all(&buf[..n]).await?;
                continue;
            }
            Poll::Pending => writer.flush().await?,
        }
        // The read future is pending, so we should wait on it.
        match read_future.await {
            Err(e) => break Err(e),
            Ok(0) => break Ok(()),
            Ok(n) => writer.write_all(&buf[..n]).await?,
        }
    };
    // Make sure that we flush any lingering data if we can.
    //
    // If there is a difference between closing and dropping, then we
    // only want to do a "proper" close if the reader closed cleanly.
    let flush_result = if loop_result.is_ok() {
        writer.close().await
    } else {
        writer.flush().await
    };
    loop_result.or(flush_result)
}

// pub trait transform_uni = ;

use std::future::Future;
use std::pin::Pin;

// pub type BoxedOperation = Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + 'static>;

pub type BoxedOperation = Box<
    dyn FnMut(
            &mut (dyn AsyncRead + Unpin + Send),
            &mut (dyn AsyncWrite + Unpin + Send),
        ) -> Pin<Box<dyn Future<Output = IoResult<()>> + Send>>
        + Send
        + 'static,
>;

fn create_func<L, R>(mut func: L) -> BoxedOperation
where
    L: FnMut(&mut (dyn AsyncRead + Unpin + Send), &mut (dyn AsyncWrite + Unpin + Send)) -> R
        + Clone
        + Send
        + 'static,
    R: Future<Output = IoResult<()>> + Send + 'static,
{
    Box::new(move |r, w| Box::pin(func(r, w)))
}

// fn identity() -> BoxedOperation
// {
// 	create_func(|r, w| copy_interactive(r,w))
// }
