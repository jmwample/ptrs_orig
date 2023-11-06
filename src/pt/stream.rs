use crate::{
    pt::transform::{BufferTransform, ReadTransform, WriteTransform},
    stream::{combine, Stream},
    wrap::WrapTransport,
    Result,
};

use tokio::io::{AsyncRead, AsyncWrite};

pub trait StreamTransport<'a, A>
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn wrap(&self, a: A) -> Result<Box<dyn Stream + 'a>>;
}

// pub fn from_transforms<'a,T1,T2, A, B>(t1:T1, t2:T2) -> impl StreamTransport<'a,A,B>
// where
// 	A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
// 	B: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
// 	T1: BufferTransform + 'a,
// 	T2: BufferTransform + 'a,
// {
// 	Box::new(FromTransforms{
// 		t1: Box::new(t1),
// 		t2: Box::new(t2),
// 	})
// }

// struct FromTransforms {
// 	t1: Box<dyn BufferTransform>,
// 	t2: Box<dyn BufferTransform>,
// }
// impl<'a,A,B> StreamTransport<'a,A,B> for FromTransforms
// where
// 	A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
// 	B: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a
// {
// 	fn wrap( &self, a: A) -> B {
// 		let (r1, w1) = tokio::io::split(a);
// 		let r_prime = ReadTransform::new(r1, self.t1.clone());
// 		let w_prime = WriteTransform::new(w1, self.t2.clone());
// 		combine(r_prime, w_prime);
// 	}
// }

impl<'a, A> StreamTransport<'a, A> for Box<dyn WrapTransport>
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn wrap(&self, a: A) -> Result<Box<dyn Stream + 'a>> {
        let (r1, w1) = tokio::io::split(a);
        let (sealer, revealer) = self.wrapper()?;
        let r_prime = revealer.reveal(Box::new(r1)); // seal outgoing stream
        let w_prime = sealer.seal(Box::new(w1)); // reveal incoming stream
        Ok(Box::new(combine(r_prime, w_prime)))
    }
}

impl<'a, A> StreamTransport<'a, A> for Box<dyn StreamTransport<'a, A>>
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn wrap(&self, a: A) -> Result<Box<dyn Stream + 'a>> {
        (**self).wrap(a)
    }
}

impl<'a, A> StreamTransport<'a, A> for &'_ dyn StreamTransport<'a, A>
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn wrap(&self, a: A) -> Result<Box<dyn Stream + 'a>> {
        (**self).wrap(a)
    }
}
