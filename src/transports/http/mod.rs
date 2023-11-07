mod duplex;
mod simplex;
mod stream;
mod wrap;

use crate::pt::copy::*;

use crate::{Configurable, Named, Result};

use futures::ready;
use http::Request;
use tokio::io::{AsyncRead, AsyncWrite};

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Http {}

impl Http {
    pub fn new() -> Self {
        Http {}
    }
}

impl Named for Http {
    fn name(&self) -> &'static str {
        "identity"
    }
}

impl Configurable for Http {
    fn with_config(self, _config: &str) -> Result<Self> {
        Ok(self)
    }
}

fn _build_http<'w, T: AsyncWrite + Unpin + 'w, B: AsRef<u8>>(_writer: T, body: B) -> Result<()> {
    let _request = Request::builder()
        .method("GET")
        .uri("https://www.rust-lang.org/")
        .header("X-Custom-Foo", "Bar")
        .body(body)
        .unwrap();

    // writer.write(request);
    Ok(())
    // let mut request = Request::builder()
    //     .uri("https://www.rust-lang.org/")
    //     .header("User-Agent", "my-awesome-agent/1.0");

    //     request = request.header("Awesome", "yes");
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

struct _Placeholder {}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::{Error, Result};

//     #[test]
//     fn test_placeholder() -> Result<()> {
//         let _p = _Placeholder {};
//         Err(Error::Other("not implemented yet".into()))
//     }
// }
