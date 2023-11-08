use ptrs::transports::identity::Identity;
use ptrs::{Result, Role, TransportBuilder};

use std::str::FromStr;

use tokio::io::{AsyncRead, AsyncWrite};

pub fn get_transport<'a, A>(_name: &str, _role: Role) -> Result<Box<dyn TransportBuilder + Send + Sync>>
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    // Transports::from_str(name)?.as_transport()
    Ok(Box::new(Identity::new()))
}

// pub async fn wrap_with<I, O>(name: &str, args: Vec<String>, role: Role, stream: I) -> Result<O>
// where
//     I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
//     O: AsyncRead + AsyncWrite + Unpin + Send + 'static,
// {
//     Transports::from_str(name)?
//         .with_config(args)?
//         .wrap(stream)
//         .await
// }

#[cfg(test)]
mod test {
    use super::*;
    use ptrs::{Named, Transport};
    use tokio::io::{split, AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixStream;

    #[tokio::test]
    async fn get_pt() -> Result<()> {
        let name = "identity";

        let transport = get_transport::<UnixStream>(name, Role::Sealer)?;
        assert_eq!(transport.name(), name);

        let (mut c, mut s) = UnixStream::pair()?;

        let mut wrapped_c = transport.build(Role::Sealer)?.wrap(&mut c)?;

        tokio::spawn(async move {
			let wrapped_s = transport.build(Role::Revealer).unwrap().wrap(&mut s).unwrap();
            let (mut r, mut w) = split(wrapped_s);
            tokio::io::copy(&mut r, &mut w).await.unwrap();
        });

        let msg = b"hello world";

        let nw = wrapped_c.write(msg).await?;
        assert_eq!(nw, msg.len());
        let nr = wrapped_c.read(&mut [0_u8; 1024]).await?;
        assert_eq!(nr, msg.len());

        Ok(())
    }
}
