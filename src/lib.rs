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
pub use pt::{copy::DuplexTransform, transform::BufferTransform, wrap::WrapTransport};
pub use stream::Stream;

#[cfg(test)]
pub(crate) mod test_utils;

use futures::Future;
use tokio::io::{AsyncRead, AsyncWrite};

pub trait Named {
    fn name(&self) -> &'static str;
}
impl Named for Box<dyn Named> {
    fn name(&self) -> &'static str {
        self.as_ref().name()
    }
}
impl Named for &'_ dyn Named {
    fn name(&self) -> &'static str {
        (*self).name()
    }
}

pub trait Configurable {
    fn with_config(self, args: &str) -> Result<Self>
    where
        Self: Sized;
}

pub trait TryConfigure {
    fn set_config(&mut self, args: &str) -> Result<()>;
}

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
pub trait Transport<'a, A>
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

// pub struct TransportInstance {
//     inner: Box<dyn for<'a> Transport<'a, Box<dyn Stream + 'a>> + Send + Sync>,
//     //    inner: Box<dyn for<'a> TransportInst<'a, Box<dyn Stream + 'a>> + Send + Sync>,

// }
// impl TransportInstance {
//     // fn new(inner: Box<dyn for<'a> TransportInst<'a, Box<dyn Stream + 'a>> + Send + Sync>) -> Self {
//     fn new(inner: Box<dyn for<'a> Transport<'a, Box<dyn Stream + 'a>> + Send + Sync>) -> Self {
//         Self { inner }
//     }
// }

// impl Named for TransportInstance {
//     fn name(&self) -> &'static str {
//         self.inner.name()
//     }
// }

// impl TryConfigure for TransportInstance {
//     fn set_config(&mut self, args: &str) -> Result<()> {
//         self.inner.set_config(args)?;
//         Ok(())
//     }
// }
// #[async_trait]
// impl<'a, A> Transport<'a, A> for TransportInstance
// where
//     A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
// {
//     async fn wrap(&self, a: A) -> Result<Box<dyn Stream + 'a>> {
//         self.inner.wrap(Box::new(a))
//     }
// }

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
pub trait Wrapping: WrapTransport + Named + TryConfigure {}
impl Named for Box<dyn Wrapping> {
    fn name(&self) -> &'static str {
        self.as_ref().name()
    }
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
pub trait Duplex<A, B>: DuplexTransform<A, B> + Named + TryConfigure
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
pub trait Transform<'a, R, W>: BufferTransform<'a, R, W> + Named + Configurable
where
    R: AsyncRead + Clone + ?Sized + 'a,
    W: AsyncWrite + Clone + ?Sized + 'a,
{
}

pub fn duplex_from_transform<'a, T, A, B>(transform: T) -> Result<Box<dyn Duplex<A, B>>>
where
    A: AsyncRead + AsyncWrite + Unpin + Clone + ?Sized + 'a,
    B: AsyncRead + AsyncWrite + Unpin + Clone + ?Sized + 'a,
    T: Transform<'a, A, B> + 'a,
{
    let _duplex: Box<dyn DuplexTransform<A, B>> =
        pt::copy::duplex_from_transform_buffer(transform)?;
    Err(Error::Other("not implemented yet".into()))
}

pub fn wrapping_from_transform<'a, T, R, W>(_transform: T) -> Result<Box<dyn Wrapping>>
where
    R: AsyncRead + Clone + ?Sized + 'a,
    W: AsyncWrite + Clone + ?Sized + 'a,
    T: Transform<'a, R, W>,
{
    Err(Error::Other("not implemented yet".into()))
}

pub fn split_stream<'s, S>(
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

pub fn split_impl<'s, S>(
    s: S,
) -> Result<(
    impl AsyncRead + Unpin + Send + Sync + 's,
    impl AsyncWrite + Unpin + Send + Sync + 's,
)>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + Sync + 's,
{
    let (r, w) = tokio::io::split(s);
    Ok((r, w))
}

pub fn split_box<'s, S>(
    s: S,
) -> Result<(
    Box<dyn AsyncRead + Unpin + Send + Sync + 's>,
    Box<dyn AsyncWrite + Unpin + Send + Sync + 's>,
)>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + Sync + 's,
{
    let (r, w) = tokio::io::split(s);
    Ok((Box::new(r), Box::new(w)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixStream;

    #[tokio::test]
    async fn splits() -> Result<()> {
        let (client, server) = UnixStream::pair()?;

        let (mut cr1, mut cw1) = split_stream(client)?;
        let (mut sr1, mut sw1) = split_stream(server)?;
        test_split_read_write(&mut cr1, &mut cw1, &mut sr1, &mut sw1).await?;

        let (client, server) = UnixStream::pair()?;
        let (mut cr2, mut cw2) = split_impl(client)?;
        let (mut sr2, mut sw2) = split_impl(server)?;
        test_split_read_write(&mut cr2, &mut cw2, &mut sr2, &mut sw2).await?;

        let (client, server) = UnixStream::pair()?;
        let (mut cr3, mut cw3) = split_box(client)?;
        let (mut sr3, mut sw3) = split_box(server)?;
        test_split_read_write(&mut cr3, &mut cw3, &mut sr3, &mut sw3).await?;
        Ok(())
    }

    async fn test_split_read_write<'a, R1, W1, R2, W2>(
        mut cr: R1,
        mut cw: W1,
        mut sr: R2,
        mut sw: W2,
    ) -> Result<()>
    where
        R1: AsyncRead + Unpin + Send + Sync + 'a,
        W1: AsyncWrite + Unpin + Send + Sync + 'a,
        R2: AsyncRead + Unpin + Send + Sync + 'a,
        W2: AsyncWrite + Unpin + Send + Sync + 'a,
    {
        let message = "hello world";

        cw.write_all(message.as_bytes()).await?;
        let mut buf = [0; 11];
        sr.read_exact(&mut buf).await?;
        assert_eq!(buf, message.as_bytes());

        let message = "goodbye";
        sw.write_all(message.as_bytes()).await?;
        let mut buf = [0; 7];
        cr.read_exact(&mut buf).await?;
        assert_eq!(buf, message.as_bytes());

        Ok(())
    }
}
