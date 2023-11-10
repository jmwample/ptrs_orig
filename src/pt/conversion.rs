use crate::{
    // pt::transform::{BufferTransform, ReadTransform, WriteTransform},
    pt::transform::BufferTransform,
    stream::{combine, Stream},
    // wrap::WrapTransport,
    Result,
    Transport, Wrapping,
    TransportBuilder, TransportInstance, Role,
};

use tokio::io::{split, AsyncRead, AsyncWrite};

pub fn from_transforms<'a, T1, T2, A, B>(t1: T1, t2: T2) -> impl Transport<'a, A>
where
    A: AsyncRead + AsyncWrite + Clone + Unpin + Send + Sync + 'a,
    B: AsyncRead + AsyncWrite + Clone + Unpin + Send + Sync + 'a,
    T1: BufferTransform<'a, A, B> + 'a,
    T2: BufferTransform<'a, B, A> + 'a,
{
    FromTransforms {
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
    t1: Box<dyn BufferTransform<'a, R1, W1> + 'a>,
    t2: Box<dyn BufferTransform<'a, R2, W2> + 'a>,
}

impl<'a, A, R1, R2, W1, W2> Transport<'a, A> for FromTransforms<'a, R1, R2, W1, W2>
where
    R1: AsyncRead + Unpin + Send + Sync + 'a,
    R2: AsyncRead + Unpin + Send + Sync + 'a,
    W1: AsyncWrite + Unpin + Send + Sync + 'a,
    W2: AsyncWrite + Unpin + Send + Sync + 'a,
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn wrap(&self, a: A) -> Result<Box<dyn Stream + 'a>> {
        let (r1, w1) = split(a);
        let (_t1, _t2) = (&self.t1, &self.t2);
        Ok(Box::new(combine(r1, w1)))

        // let r_prime = ReadTransform::new(r1,  t1);
        // let w_prime = WriteTransform::new( w1,  t2);
        // Ok(Box::new(combine(r_prime, w_prime)))
    }
}

impl TransportBuilder for Box<dyn Wrapping> {
    fn build(&self, r: &Role) -> Result<TransportInstance>{
        match r {
            Role::Sealer => {
                Ok(TransportInstance::new(Box::new(self.sealer()?)))
            }
            Role::Revealer => {
                Ok(TransportInstance::new(Box::new(self.revealer()?)))
            }
        }
    }
}

impl<'a, A> Transport<'a, A> for Box<dyn Transport<'a, A>>
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn wrap(&self, a: A) -> Result<Box<dyn Stream + 'a>> {
        (**self).wrap(a)
    }
}

impl<'a, A> Transport<'a, A> for &'_ dyn Transport<'a, A>
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn wrap(&self, a: A) -> Result<Box<dyn Stream + 'a>> {
        (**self).wrap(a)
    }
}
