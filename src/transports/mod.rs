pub mod base64;
pub mod ecdh_ed25519;
pub mod hex_encoder;
pub mod http;
pub mod prefix_tls_rec_frag;
pub mod reverse;
pub mod ss_format;

pub mod identity;

// pub enum Transports {
//     Identity(Box<dyn AsTransport>),
//     Reverse(Box<dyn AsTransport>),
//     HexEncoder(Box<dyn AsTransport>),
//     // Http(http::Http),
//     // PrefixTlsRecFrag(prefix_tls_rec_frag::PrefixTlsRecFrag),
//     // SsFormat(ss_format::SsFormat),
//     // EcdhEd25519(ecdh_ed25519::EcdhEd25519),
//     // Base64(base64::Base64),

//     // Other(Box<dyn TransportBuilder>),
//     // OtherStreamHandler(Box<dyn StreamHandler>),
// }

// impl FromStr for Transports {
//     type Err = crate::Error;

//     fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
//         match s {
//             "" | "identity" => Ok(Transports::Identity(Box::new(identity::Identity::new()))),
//             "reverse" => Ok(Transports::Reverse(Box::new(reverse::Reverse::new()))),
//             "hex" => Ok(Transports::HexEncoder(Box::new(
//                 hex_encoder::HexEncoder::new(),
//             ))),
//             // "http" => Ok(Transports::Http(http::Http::new())),
//             // "prefix_tls_rec_frag" => Ok(Transports::PrefixTlsRecFrag(prefix_tls_rec_frag::PrefixTlsRecFrag::new())),
//             // "ss_format" => Ok(Transports::SsFormat(ss_format::SsFormat::new())),
//             // "ecdh_ed25519" => Ok(Transports::EcdhEd25519(ecdh_ed25519::new()))
//             // "base64" => Ok(Transports::Base64(base64::Base64::new())),
//             // "water" =>
//             // proteus =>
//             _ => Err(std::io::Error::new(std::io::ErrorKind::Other, "not implemented yet").into()),
//         }
//     }
// }

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
