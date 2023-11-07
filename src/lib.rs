#![feature(trait_alias)]
#![doc = include_str!("../README.md")]

mod errors;
mod other_copy;

pub use errors::{Error, Result};

pub mod stream;
pub mod sync;
pub mod transports;

mod pt;
pub use pt::*;
use pt::{
    copy::DuplexTransform, stream::StreamTransport, transform::BufferTransform, wrap::WrapTransport,
};

#[cfg(test)]
pub(crate) mod test_utils;

use tokio::io::{AsyncRead, AsyncWrite};

pub trait Named {
    fn name(&self) -> &'static str;
}

pub trait Configurable {
    fn with_config(self, args: &str) -> Result<Self>
    where
        Self: Sized;
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
/// [`shutdown()`]: crate::io::AsyncWriteExt::shutdown
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
pub trait Transport<'a, A>: StreamTransport<'a, A> + Named + Configurable
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
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
/// [`shutdown()`]: crate::io::AsyncWriteExt::shutdown
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
pub trait Wrapping: WrapTransport + Named + Configurable {}

/// Copies data in one direction from `a` to `b`, applying the transform as it goes.
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
/// [`shutdown()`]: crate::io::AsyncWriteExt::shutdown
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
pub trait Duplex<A, B>: DuplexTransform<A, B> + Named + Configurable
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
}

/// Copies data in one direction from `a` to `b`, applying the transform as it goes.
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
/// [`shutdown()`]: crate::io::AsyncWriteExt::shutdown
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
pub trait Transform: BufferTransform + Named + Configurable {}

pub fn duplex_from_transform<T, A, B>(transform: T) -> Result<Box<dyn Duplex<A, B>>>
where
    T: Transform,
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    let _duplex: Box<dyn DuplexTransform<A, B>> =
        pt::copy::duplex_from_transform_buffer(transform)?;
    Err(Error::Other("not implemented yet".into()))
}

pub fn wrapping_from_transform<T>(_transform: T) -> Result<Box<dyn Wrapping>>
where
    T: Transform,
{
    Err(Error::Other("not implemented yet".into()))
}

pub fn split<'s, S>(
    s: S,
) -> Result<(
    Box<dyn stream::ReadHalf + 's>,
    Box<dyn stream::WriteHalf + 's>,
)>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + Sync + 's,
{
    let (r, w) = tokio::io::split(s);
    Ok((Box::new(r), Box::new(w)))
}
