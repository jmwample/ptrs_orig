//! # PT
//!

pub use crate::pt::{copy::DuplexTransform, transform::BufferTransform, wrap::WrapTransport};
use crate::{Configurable, Error, Named, Result, TryConfigure};

use tokio::io::{AsyncRead, AsyncWrite};

pub(crate) mod copy_buffer;

/// copy based pluggable transports construction tools.
pub mod copy;

/// Buffer transform based pluggable transports construction tools.
pub mod transform;

/// Wrapper based pluggable transports construction tools.
pub mod wrap;

mod conversion;
pub use conversion::*;

/// Takes an AsyncRead/AsyncWrite and returns a future that constructs an object
/// to automatically encode/decode when the wrapper is read from / written to.
///
/// This function returns a future that will read from both streams, writing any
/// data read to the opposing stream. This happens in both directions
/// concurrently.
///
/// If an EOF is observed on one stream, [`shutdown()`] will be invoked on the
/// other, and reading from that stream will stop. Copying of data in the other
/// direction will continue.
///
/// The future will complete successfully once both directions of communication
/// has been shut down. A direction is shut down when the reader reports EOF, at
/// which point [`shutdown()`] is called on the corresponding writer. When
/// finished, it will return a tuple of the number of bytes copied from a to b
/// and the number of bytes copied from b to a, in that order.
///
/// [`shutdown()`]: tokio::io::AsyncWriteExt::shutdown
///
/// # Errors
///
/// The future will immediately return an error if any IO operation on `a` or
/// `b` returns an error. Some data read from either stream may be lost (not
/// written to the other stream) in this case.
///
/// # Return value
///
/// Returns a tuple of bytes copied `a` to `b` and bytes copied `b` to `a`.
pub trait Wrapping: WrapTransport + Named + TryConfigure {}
impl Named for Box<dyn Wrapping> {
    fn name(&self) -> String {
        self.as_ref().name()
    }
}

/// Copies data both directions `a -> b` and `b -> a`. The data is encoded/decoded as it goes.
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
/// Returns a count of bytes copied `a` to `b`.
pub trait Duplex<A, B>: DuplexTransform<A, B> + Named + TryConfigure
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
}

/// Copies data in one direction from `a -> b`, reading into buffer and applying the transform as it goes.
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
/// Returns a count of bytes copied `a` to `b`.
pub trait Transform<'a, R, W>: BufferTransform<'a, R, W> + Named + Configurable
where
    R: AsyncRead + Clone + ?Sized + 'a,
    W: AsyncWrite + Clone + ?Sized + 'a,
{
}

/// Convert two buffer transforms into a duplex based transport.
pub fn duplex_from_transform<'a, T, A, B>(transform: T) -> Result<Box<dyn Duplex<A, B>>>
where
    A: AsyncRead + AsyncWrite + Unpin + Clone + ?Sized + 'a,
    B: AsyncRead + AsyncWrite + Unpin + Clone + ?Sized + 'a,
    T: Transform<'a, A, B> + 'a,
{
    let _duplex: Box<dyn DuplexTransform<A, B>> = copy::duplex_from_transform_buffer(transform)?;
    Err(Error::Other("not implemented yet".into()))
}

/// Convert two buffer transforms into a Wrapping transport.
pub fn wrapping_from_transform<'a, T, R, W>(_transform: T) -> Result<Box<dyn Wrapping>>
where
    R: AsyncRead + Clone + ?Sized + 'a,
    W: AsyncWrite + Clone + ?Sized + 'a,
    T: Transform<'a, R, W>,
{
    Err(Error::Other("not implemented yet".into()))
}
