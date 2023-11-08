use ptrs::transports::identity::Identity;
use ptrs::{Result, Role, TransportBuilder};

// use std::str::FromStr;

pub fn get_transport(_name: &str, _role: &Role) -> Result<Box<dyn TransportBuilder + Send + Sync>> {
    // Transports::from_str(name)?.as_transport()
    Ok(Box::new(Identity::new()))
}

#[cfg(test)]
mod test {
    use super::*;
    use ptrs::Transport;
    use tokio::io::{split, AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixStream;

    #[tokio::test]
    async fn get_pt() -> Result<()> {
        let name = "identity";

        let transport = get_transport(name, &Role::Sealer)?;
        assert_eq!(transport.name(), name);

        let (c, s) = UnixStream::pair()?;

        let mut wrapped_c = transport.build(&Role::Sealer)?.wrap(Box::new(c))?;

        tokio::spawn(async move {
            let wrapped_s = transport
                .build(&Role::Revealer)
                .unwrap()
                .wrap(Box::new(s))
                .unwrap();
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
