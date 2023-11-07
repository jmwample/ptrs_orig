use crate::{
    stream::{combine, Stream},
    wrap::WrapTransport,
    pt::transform::{BufferTransform, ReadTransform, WriteTransform},
    Result,
};

use tokio::io::{AsyncRead, AsyncWrite};

pub trait StreamTransport<'a, A>
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn wrap(&self, a: &'a mut A) -> Result<Box<dyn Stream + 'a>>;
}

pub fn from_transforms<'a,T1,T2, A, B>(t1:T1, t2:T2) -> impl StreamTransport<'a,A>
where
	A: AsyncRead + AsyncWrite + Clone + Unpin + Send + Sync + 'a,
	B: AsyncRead + AsyncWrite + Clone + Unpin + Send + Sync + 'a,
	T1: BufferTransform<'a,A,B> + 'a,
	T2: BufferTransform<'a,B,A> + 'a,
{
	FromTransforms{
		t1: Box::new(t1),
		t2: Box::new(t2),
	}
}

struct FromTransforms<'a, R1, R2, W1, W2>
where
    R1: AsyncRead + Unpin + Send + Sync + 'a,
	R2: AsyncRead + Unpin + Send + Sync + 'a,
    W1: AsyncWrite + Unpin + Send + Sync + 'a,
	W2: AsyncWrite + Unpin + Send + Sync + 'a,
{
	t1: Box<dyn BufferTransform<'a,R1,W1> + 'a>,
	t2: Box<dyn BufferTransform<'a,R2,W2> + 'a>,
}

impl<'a,R1,R2,W1,W2> FromTransforms<'a, R1, R2, W1, W2>
where
    R1: AsyncRead + Unpin + Send + Sync + 'a,
	R2: AsyncRead + Unpin + Send + Sync + 'a,
    W1: AsyncWrite + Unpin + Send + Sync + 'a,
	W2: AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn take_transforms(self) -> Result<(Box<dyn BufferTransform<'a,R1,W1> + 'a>, Box<dyn BufferTransform<'a,R2,W2> + 'a>)> {
        Ok((self.t1, self.t2))
    }
}


impl<'a,A,R1,R2,W1,W2> StreamTransport<'a, A> for FromTransforms<'a, R1, R2, W1, W2>
where
    R1: AsyncRead + Unpin + Send + Sync + 'a,
	R2: AsyncRead + Unpin + Send + Sync + 'a,
    W1: AsyncWrite + Unpin + Send + Sync + 'a,
	W2: AsyncWrite + Unpin + Send + Sync + 'a,
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn wrap(&self, a: &'a mut A) -> Result<Box<dyn Stream + 'a>>{
		let (mut r1, mut w1) = crate::split(a)?;
        let (mut t1, mut t2) = self.take_transforms()?;

		let r_prime = ReadTransform::new(r1,  t1);
		let w_prime = WriteTransform::new( w1,  t2);
		Ok(Box::new(combine(r_prime, w_prime)))
	}
}

impl<'a, A> StreamTransport<'a, A> for Box<dyn WrapTransport>
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn wrap(&self, a: &'a mut A) -> Result<Box<dyn Stream + 'a>> {
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
    fn wrap(&self, a: &'a mut A) -> Result<Box<dyn Stream + 'a>> {
        (**self).wrap(a)
    }
}

impl<'a, A> StreamTransport<'a, A> for &'_ dyn StreamTransport<'a, A>
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn wrap(&self, a: &'a mut A) -> Result<Box<dyn Stream + 'a>> {
        (**self).wrap(a)
    }
}
