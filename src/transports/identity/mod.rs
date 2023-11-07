use crate::pt::copy::*;
use crate::{Configurable, Named, Result};
use futures::ready;
use tokio::io::{AsyncRead, AsyncWrite};

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

mod duplex;
mod simplex;
mod stream;
mod wrap;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Identity {}

impl Identity {
    pub fn new() -> Self {
        Identity {}
    }
}

impl Named for Identity {
    fn name(&self) -> &'static str {
        "identity"
    }
}

impl Configurable for Identity {
    fn with_config(self, _config: &str) -> Result<Self> {
        Ok(self)
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::tests::duplex_end_to_end_1_MB;
    use crate::{pt::wrap::*, test_utils::init_subscriber};

    use futures::try_join;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn simplex_to_duplex() {
        let encode = Identity::new();
        let decode = Identity::new();

        let t = duplex_from_simplices(encode, decode);

        let (mut source, mut plaintext) = tokio::net::UnixStream::pair().unwrap();
        let (mut ciphertext, mut echo) = tokio::net::UnixStream::pair().unwrap();

        let (up, down) =
            duplex_end_to_end_1_MB(&mut source, &mut plaintext, &mut ciphertext, &mut echo, t)
                .await
                .unwrap();
        assert_eq!(up, 1024 * 1024);
        assert_eq!(down, 1024 * 1024);
    }

    #[tokio::test]
    async fn duplex() {
        init_subscriber();

        let (mut source, mut plaintext) = tokio::net::UnixStream::pair().unwrap();
        let (mut ciphertext, mut echo) = tokio::net::UnixStream::pair().unwrap();

        let (up, down) = duplex_end_to_end_1_MB(
            &mut source,
            &mut plaintext,
            &mut ciphertext,
            &mut echo,
            Identity::new(),
        )
        .await
        .unwrap();
        assert_eq!(up, 1024 * 1024);
        assert_eq!(down, 1024 * 1024);
    }

    ///                __              __
    ///                |     (Sealer)    |
    ///         write  | reader [ read ] |===============> echo
    ///                |__             __|                  ||
    ///         __             __                           ||
    ///        |    (Revealer)   |                          ||
    ///        | [ read ] reader | write <===================
    ///        |__             __|
    ///
    #[tokio::test]
    async fn wrap_transport() {
        let (sealer, revealer) = Identity::default().wrapper().unwrap();
        let (mut client, mut server) = tokio::net::UnixStream::pair().unwrap();

        let server_task = tokio::spawn(async move {
            let (r, w) = server.split();
            let mut wrapped_w = sealer.seal(Box::new(w));
            let mut wrapped_r = revealer.reveal(Box::new(r));
            tokio::io::copy(&mut wrapped_r, &mut wrapped_w)
                .await
                .unwrap();
        });

        let client_task = tokio::spawn(async move {
            let (mut cr, mut cw) = client.split();
            let nw = cw.write(&[0_u8; 1024]).await.unwrap();
            assert_eq!(nw, 1024);

            let mut buf = [0_u8; 1024];
            let nr = cr.read(&mut buf).await.unwrap();
            assert_eq!(nr, 1024);
        });

        try_join!(client_task, server_task).unwrap();
    }
}
