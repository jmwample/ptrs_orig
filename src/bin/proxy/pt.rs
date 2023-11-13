use ptrs::{transports::identity, Result, Role, Stream, Transport};
use tokio::io::{AsyncRead, AsyncWrite};

// use std::str::FromStr;

pub fn get_transport(name: &str, role: Role) -> Result<TransportBuilder> {
    Ok(TransportBuilder {
        name: name.into(),
        role,
        config: "".into(),
    })
}

#[derive(Clone, PartialEq)]
pub struct TransportBuilder {
    pub name: String,
    pub role: Role,
    pub config: String,
}

impl TransportBuilder {
    pub async fn wrap<'a, A>(self, a: A) -> Result<Box<dyn Stream + 'a>>
    where
        A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
    {
        Ok(self.build::<A>()?.wrap(a).await?)
    }

    fn build<'a, A>(self) -> Result<Box<identity::Identity>> {
        Ok(Box::new(identity::Identity::default()))
    }
}

// pub struct Transport<'a, A> {
//     pt: Box<dyn pt<'a,A>>
// }

// impl<'a, A> Transport<'a, A>
// where
//     A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a
// {
//     pub async fn wrap(&self, a: A)-> Result<Box<dyn Stream + 'a>> {
//         self.pt.wrap(a)
//     }
// }

// #[cfg(test)]
// mod test {
//     use super::*;
//     use ptrs::Transport;
//     use tokio::io::{split, AsyncReadExt, AsyncWriteExt};
//     use tokio::net::UnixStream;

//     #[tokio::test]
//     async fn get_pt() -> Result<()> {
//         let name = "identity";

//         let transport = get_transport(name, &Role::Sealer)?;

//         let (c, s) = UnixStream::pair()?;

//         let mut wrapped_c = transport.build(&Role::Sealer)?.wrap(Box::new(c))?;

//         tokio::spawn(async move {
//             let wrapped_s = transport
//                 .build(&Role::Revealer)
//                 .unwrap()
//                 .wrap(Box::new(s))
//                 .unwrap();
//             let (mut r, mut w) = split(wrapped_s);
//             tokio::io::copy(&mut r, &mut w).await.unwrap();
//         });

//         let msg = b"hello world";

//         let nw = wrapped_c.write(msg).await?;
//         assert_eq!(nw, msg.len());
//         let nr = wrapped_c.read(&mut [0_u8; 1024]).await?;
//         assert_eq!(nr, msg.len());

//         Ok(())
//     }
// }

// // TEMPORARY

// use ptrs::{Role, Result, Error};

// pub trait Transport = for<'a, A> ptrs::Transport<'a, A> + Send + Sync + Sized;

// pub trait TransportBuilder{
//     fn build(&self, role: ptrs::Role) -> impl Transport;
// }

// pub fn get_transport<'a>(name: &str, _role: &Role) -> Result<Box<dyn TransportBuilder + Send + Sync + 'a>> {
//     Err(Error::Other("not implemented".into()))
// }
