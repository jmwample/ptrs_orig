#![feature(trait_alias)]
#![doc = include_str!("../doc/crate.md")]

mod errors;
mod other_copy;

pub use errors::{Error, Result};

/// Tools and abstractions for I/O such that they can be used interchangeably.
///
/// # Stream I/O Abstractions and Operations
///
/// The `Stream` trait is an abstraction over I/O interfaces requiring only that
/// they implements [AsyncRead], [AsyncWrite], and are safe to send between
/// threads.
///
/// Streams can be split into separate read and write halves using one of the
/// `split*` functions ([split_stream](crate::stream::split_stream),
/// [split_impl](crate::stream::split_impl),
/// [split](crate::stream::split)). The halves can then be combined back
/// into a single stream using [combine](crate::stream::combine).
///
/// ```
/// # use tokio::io::{duplex,AsyncWriteExt};
/// # use ptrs::stream::split_stream;
/// # #[tokio::main]
/// # async fn main() -> ptrs::Result<()> {
/// let (a, mut b) = duplex(128);
/// let (mut ra, mut wa) = split_stream(a)?;
/// # tokio::spawn(async move {
/// #     std::thread::sleep(std::time::Duration::from_millis(100));
/// #     b.write_all(b"hello world").await.unwrap();
/// #     drop(b);
/// # });
/// let res = tokio::io::copy(&mut ra,&mut wa).await;
/// # Ok(())
/// # }
/// ```
///
/// Unlike `tokio::io::ReadHalf.unsplit()`, [combine](crate::stream::combine)
/// does not require that the halves originated from the same original object.
///
/// ```
/// # use tokio::io::duplex;
/// # use ptrs::stream::{split_stream,combine};
/// # #[tokio::main]
/// # async fn main() -> ptrs::Result<()> {
/// let (a, b) = duplex(128);
/// let (ra, wa) = split_stream(a)?;
/// let (rb, wb) = split_stream(b)?;
/// let x = combine(ra, wb);
/// let y = combine(rb, wa);
/// # Ok(())
/// # }
/// ```
pub mod stream;

/// [UNDER CONSTRUCTION] Synchronous versions of the pluggable transport interface constructions.
pub mod sync;

/// Example transport used for motivating features in the pluggable transport interface.
pub mod transports;

/// Pluggable transport interface constructions.
pub mod pt;
pub use pt::{copy, transform, wrap};

pub use stream::Stream;

#[cfg(test)]
pub(crate) mod test_utils;

use futures::Future;
use tokio::io::{AsyncRead, AsyncWrite};

/// A trait indicating that an object has a name in the transport context.
pub trait Named {
    fn name(&self) -> String;
}
impl Named for Box<dyn Named> {
    fn name(&self) -> String {
        self.as_ref().name()
    }
}
impl Named for &'_ dyn Named {
    fn name(&self) -> String {
        (*self).name()
    }
}

/// Builder pattern trait indicating that a type can be configured with a string.
pub trait Configurable {
    fn with_config(self, args: &str) -> Result<Self>
    where
        Self: Sized;
}

/// Mutator trait indicating that a type can be configured with a string.
pub trait TryConfigure {
    fn set_config(&mut self, args: &str) -> Result<()>;
}

/// Directional indicators for pluggable transport builders
#[derive(Clone, PartialEq)]
pub enum Role {
    /// Plaintext -> Ciphertext transformation
    Sealer,

    /// Ciphertext -> Plaintext transformation
    Revealer,
}

/// Copies data in both directions between `a` and `b`, encoding/decoding as it goes.
///
/// This function returns a future that will read from both streams,
/// writing any data read to the opposing stream.
/// This happens in both directions concurrently.
///
/// If an EOF is observed on one stream, [`shutdown()`] will be invoked on
/// the other, and reading from that stream will stop. Copying of data in
/// the other direction will continue.
///
/// The future will complete successfully once both directions of communication has been shut down.
/// A direction is shut down when the reader reports EOF,
/// at which point [`shutdown()`] is called on the corresponding writer. When finished,
/// it will return a tuple of the number of bytes copied from a to b
/// and the number of bytes copied from b to a, in that order.
///
/// [`shutdown()`]: tokio::io::AsyncWriteExt::shutdown
///
/// # Errors
///
/// The future will immediately return an error if any IO operation on `a`
/// or `b` returns an error. Some data read from either stream may be lost (not
/// written to the other stream) in this case.
///
/// # Return value
///
/// Returns a tuple of bytes copied `a` to `b` and bytes copied `b` to `a`.
pub trait Transport<'a, A>: Named
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn wrap(&self, a: A) -> impl Future<Output = Result<Box<dyn Stream + 'a>>>;
}

pub trait TransportInst<'a, A>: Named + TryConfigure + Transport<'a, A>
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
}

pub trait TransportBuilder<'a, S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn build(&'a self, role: Role) -> impl Transport<'a, S> + Send + Sync + 'a;
}

// pub struct TransportInstance {
//     pub inner: Box<dyn for<'a> Transport<'a, Box<dyn Stream + 'a>> + Send + Sync>,
//     //    inner: Box<dyn for<'a> TransportInst<'a, Box<dyn Stream + 'a>> + Send + Sync>,

// }

// impl TransportInstance {
//     // fn new(inner: Box<dyn for<'a> TransportInst<'a, Box<dyn Stream + 'a>> + Send + Sync>) -> Self {
//     fn new(inner: Box<dyn for<'a> Transport<'a, Box<dyn Stream + 'a>> + Send + Sync>) -> Self {
//         Self { inner }
//     }

//     fn wrap_inner<'a, A>(&self, a: A) -> Result<Box<dyn Stream + 'a>>
//     where
//         A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
//     {
//         self.inner.wrap(Box::new(a))
//     }
// }

// impl Named for TransportInstance {
//     fn name(&self) -> String {
//         self.inner.name()
//     }
// }

// impl TryConfigure for TransportInstance {
//     fn set_config(&mut self, args: &str) -> Result<()> {
//         self.inner.set_config(args)?;
//         Ok(())
//     }
// }

// impl<'a, A> Transport<'a, A> for TransportInstance
// where
//     A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
// {
//     fn wrap(&self, a: A) -> impl Future<Output=Result<Box<dyn Stream + 'a>>> {
//         self.wrap_inner(a)
//     }
// }
