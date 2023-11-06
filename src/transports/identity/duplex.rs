use crate::pt::copy::*;
use crate::pt::copy_buffer::CopyBuffer;
use futures::{future::poll_fn, ready};
use tokio::io::{AsyncRead, AsyncWrite};

use async_trait::async_trait;

use std::task::Poll;

use super::{transfer_one_direction, Identity};

#[async_trait]
impl<A, B> DuplexTransform<A, B> for Identity
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + Send + Sync + ?Sized,
{
    async fn copy_bidirectional<'a, 'b>(
        &self,
        a: &'a mut A,
        b: &'b mut B,
    ) -> std::result::Result<(u64, u64), std::io::Error>
    where
        A: AsyncRead + AsyncWrite + Unpin,
        B: AsyncRead + AsyncWrite + Unpin,
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
