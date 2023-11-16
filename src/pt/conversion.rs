use crate::{
    // pt::transform::{BufferTransform, ReadTransform, WriteTransform},
    pt::transform::BufferTransform,
    stream::{combine, Stream},
    Named, //Role, TransportInst, TransportBuilder,
    Result,
    // pt::Wrapping,
    // wrap::WrapTransport,
    Transport,
};

use tokio::io::{split, AsyncRead, AsyncWrite};

/// Build a transport from a pair of transforms
pub fn from_transforms<'a, T1, T2, A, B>(t1: T1, t2: T2, name: String) -> impl Transport<'a, A>
where
    A: AsyncRead + AsyncWrite + Clone + Unpin + Send + Sync + 'a,
    B: AsyncRead + AsyncWrite + Clone + Unpin + Send + Sync + 'a,
    T1: BufferTransform<'a, A, B> + 'a,
    T2: BufferTransform<'a, B, A> + 'a,
{
    FromTransforms {
        t1: Box::new(t1),
        t2: Box::new(t2),
        name,
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
    name: String,
}

impl<'a, R1, R2, W1, W2> Named for FromTransforms<'a, R1, R2, W1, W2>
where
    R1: AsyncRead + Unpin + Send + Sync + 'a,
    R2: AsyncRead + Unpin + Send + Sync + 'a,
    W1: AsyncWrite + Unpin + Send + Sync + 'a,
    W2: AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn name(&self) -> String {
        self.name.clone()
    }
}

// #[async_trait]
impl<'a, A, R1, R2, W1, W2> Transport<'a, A> for FromTransforms<'a, R1, R2, W1, W2>
where
    R1: AsyncRead + Unpin + Send + Sync + 'a,
    R2: AsyncRead + Unpin + Send + Sync + 'a,
    W1: AsyncWrite + Unpin + Send + Sync + 'a,
    W2: AsyncWrite + Unpin + Send + Sync + 'a,
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    async fn wrap(&self, a: A) -> Result<Box<dyn Stream + 'a>> {
        let (r1, w1) = split(a);
        let (_t1, _t2) = (&self.t1, &self.t2);
        Ok(Box::new(combine(r1, w1)))

        // let r_prime = ReadTransform::new(r1,  t1);
        // let w_prime = WriteTransform::new( w1,  t2);
        // Ok(Box::new(combine(r_prime, w_prime)))
    }
}

// impl TransportBuilder for Box<dyn Wrapping> {
//     fn build(&self, r: &Role) -> Result<TransportInst>{
//         match r {
//             Role::Sealer => {
//                 Ok(TransportInst::new(Box::new(self.sealer()?)))
//             }
//             Role::Revealer => {
//                 Ok(TransportInst::new(Box::new(self.revealer()?)))
//             }
//         }
//     }
// }

// #[async_trait]
// impl<'a, A> Transport<'a, A> for Box<dyn Transport<'a, A>>
// where
//     A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
// {
//     async fn wrap(&self, a: A) -> Result<Box<dyn Stream + 'a>> {
//         (**self).wrap(a)
//     }
// }

// #[async_trait]
// impl<'a, A> Transport<'a, A> for &'_ dyn Transport<'a, A>
// where
//     A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
// {
//     async fn wrap(&self, a: A) -> Result<Box<dyn Stream + 'a>> {
//         (**self).wrap(a)
//     }
// }
