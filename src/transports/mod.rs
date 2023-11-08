pub mod base64;
pub mod ecdh_ed25519;
pub mod hex_encoder;
pub mod http;
pub mod prefix_tls_rec_frag;
pub mod reverse;
pub mod ss_format;

pub mod identity;

use std::str::FromStr;

use crate::{pt::wrap::WrapTransport, stream::Stream, Error, Result, Transport};

use tokio::io::{AsyncRead, AsyncWrite};

pub enum Transports {
    Identity,
    Reverse,
    // HexEncoder,
    // Http,
    // PrefixTlsRecFrag,
    // SsFormat,
    // EcdhEd25519,
    Base64,
    // Other(Box<dyn TransportBuilder>),
    // OtherStreamHandler(Box<dyn StreamHandler>),
}

impl FromStr for Transports {
    type Err = crate::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "" | "identity" => Ok(Transports::Identity),
            "reverse" => Ok(Transports::Reverse),
            // "hex" => Ok(Transports::HexEncoder),
            "base64" => Ok(Transports::Base64),
            _ => Err(std::io::Error::new(std::io::ErrorKind::Other, "not implemented yet").into()),
        }
    }
}

impl Transports {
    pub fn build<'a, A>(&self) -> Box<dyn Transport<'a, A> + 'a>
    where
        A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
    {
        match self {
            Transports::Identity => Box::new(identity::Identity::new()),
            Transports::Reverse => Box::new(reverse::Reverse::new()),
            Transports::Base64 => {
                let wt: Box<dyn WrapTransport> = Box::new(base64::Base64Builder::default());
                Box::new(wt)
            } // Transports::HexEncoder => Box::new(hex_encoder::HexEncoder::new()),
        }
    }
}

struct NullTransport {}

impl NullTransport {
    fn new() -> Self {
        Self {}
    }
}

impl Default for NullTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, A> Transport<'a, A> for NullTransport
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn wrap(&self, _r: A) -> Result<Box<dyn Stream + 'a>> {
        Err(Error::NullTransport)
    }
}

#[cfg(test)]
mod test {
    use crate::Result;

    #[test]
    fn transports_interface() -> Result<()> {
        // let name_set = vec!["identity", "hex", "reverse"];
        // for name in name_set {
        //     let mut tr = Transports::from_str(name)?;
        //     let t = tr.as_transport()?;
        //     assert_eq!(t.name(), name);
        // }
        Ok(())
    }
}
