use tokio::io::{AsyncRead, AsyncWrite};

use std::io;
use std::task::{Context, Poll};

use super::{transfer_one_direction, Identity};
use crate::pt::copy::*;

impl<A, B> SimplexTransform<A, B> for Identity
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    fn transfer_one_direction(
        &self,
        cx: &mut Context<'_>,
        state: &mut TransferState,
        r: &mut A,
        w: &mut B,
    ) -> Poll<io::Result<u64>> {
        transfer_one_direction(cx, state, r, w)
    }
}
