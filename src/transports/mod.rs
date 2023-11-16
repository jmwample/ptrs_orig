pub mod identity;

pub mod base64;
pub mod hex_encoder;
pub mod http;
pub mod reverse;
pub mod rustls;
// pub mod proteus;

pub mod ecdh_ed25519;
pub mod prefix_tls_rec_frag;
pub mod ss_format;

use crate::{stream::Stream, Error, Named, Result, Transport, TryConfigure};

use tokio::io::{AsyncRead, AsyncWrite};

use std::str::FromStr;

pub enum Transports {
    Identity,
    Reverse,
    // HexEncoder,
    // Http,
    // Rustls,

    // PrefixTlsRecFrag,
    // SsFormat,
    // EcdhEd25519,
    Base64,
    // Other(Box<dyn TransportBuilder>),
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

// impl Transports {
//     pub fn builder<'a>(&self) -> Box<dyn TransportBuilder + Send + Sync + 'a>
//     {
//         match self {
//             Transports::Identity => Box::<identity::Identity>::default(),
//             Transports::Reverse => Box::<reverse::Builder>::default(),
//             Transports::Base64 => Box::<Base64Builder>::default(),
//             // Transports::HexEncoder => Box::<hex_encoder::HexEncoder>::default()),
//         }
//     }
// }

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

// #[async_trait]
impl<'a, A> Transport<'a, A> for NullTransport
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    async fn wrap(&self, _r: A) -> Result<Box<dyn Stream + 'a>> {
        Err(Error::NullTransport)
    }
}
impl Named for NullTransport {
    fn name(&self) -> String {
        "null".into()
    }
}
impl TryConfigure for NullTransport {
    fn set_config(&mut self, _config: &str) -> Result<()> {
        Ok(())
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
