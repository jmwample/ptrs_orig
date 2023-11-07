// use crate::pt::{stream::Transform, Transport};
use crate::{Configurable, Named, Result};
use std::io::{BufReader, Read, Write};

use tokio::io::{AsyncRead, AsyncReadExt};

pub const NAME: &str = "reverse";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Reverse {}

impl Reverse {
    pub fn new() -> Self {
        Reverse {}
    }
}

impl Named for Reverse {
    fn name(&self) -> &'static str {
        NAME
    }
}

impl Configurable for Reverse {
    fn with_config(self, _config: &str) -> Result<Self> {
        Ok(self)
    }
}

pub async fn reverse<T: AsyncRead+Unpin>(mut r: T, mut w: &mut [u8]) -> Result<usize> {
    let mut buf = vec![0_u8; 1024];
    let nr = r.read(&mut buf).await?;
    // println!("n: {} {:?}", nr, &buf[..nr]);
    let processed: Vec<u8> = buf[..nr].iter().copied().rev().collect();

    let nw = w.write(&processed[..nr])?;
    // println!("processed: {:?}", &processed[..nw]);

    Ok(nw)
}

pub fn reverse_sync(incoming: &mut dyn Read, outgoing: &mut dyn Write) -> Result<u64> {
    let mut readbuf = BufReader::new(incoming);

    let mut buf = vec![0_u8; 1024];
    let nr = readbuf.read(&mut buf)?;
    // println!("n: {} {:?}", nr, &buf[..nr]);
    let processed: Vec<u8> = buf[..nr].iter().copied().rev().collect();

    let nw = outgoing.write(&processed[..nr])?;
    // println!("processed: {:?}", &processed[..nw]);

    Ok(nw as u64)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn traits() {
        // ensure we get the right name
        let reverse = Reverse::new();
        assert_eq!(reverse.name(), NAME);

        // config does nothing for this transport
        let config = "config";
        let reverse_n = Reverse::new().with_config(config).unwrap();
        assert_eq!(reverse, reverse_n);
    }

    #[tokio::test]
    async fn reverse_transform_async() -> Result<()> {
        use tokio::io::BufReader;

        let message = b"hello world";
        let mut msg = BufReader::new(&message[..]);

        let mut out = vec![0_u8; 1024];
        let nw = reverse(&mut msg, &mut out).await?;

        assert_eq!(std::str::from_utf8(&out[..nw]).unwrap(), "dlrow olleh");

        let mut msg = BufReader::new(&out[..nw]);
        let mut f = vec![0_u8; 1024];
        let nw = reverse(&mut msg, &mut f).await?;

        assert_eq!(nw, message.len());
        assert_eq!(f[..nw], message[..nw]);

        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_reverse() -> Result<()> {
        use std::os::unix::net::UnixStream;

        let (mut client_host, mut client_wasm) = UnixStream::pair()?;
        let (mut wasm_remote, mut remote) = UnixStream::pair()?;

        let buf = b"hello world";

        let transport_result = {
            client_host.write_all(buf)?;
            reverse_sync(&mut client_wasm, &mut wasm_remote)
        };

        let mut out = vec![0_u8; 1024];
        let nr = remote.read(&mut out)?;

        assert!(transport_result.is_ok());
        let n = transport_result? as usize;
        assert_eq!(n, buf.len());
        assert_eq!(n, nr);
        assert_eq!(std::str::from_utf8(&out[..n]).unwrap(), "dlrow olleh");
        Ok(())
    }
}
