use crate::{
    wrap::{Reveal, Seal, WrapTransport, Wrapper},
    Configurable, Named, Result,
    Role, TransportBuilder, TransportInstance,
};

use tokio::io::{AsyncRead, AsyncWrite};

use base64::engine::general_purpose;

struct Config {
    _engine_config: general_purpose::GeneralPurposeConfig,
}

const NAME: &str = "base64";

pub struct Base64 {
    _engine: general_purpose::GeneralPurpose,
}

#[derive(Default)]
pub struct Base64Builder {
    _config: Option<Config>,
}

// impl Transport for Base64Builder {}
impl Named for Base64Builder {
    fn name(&self) -> &'static str {
        NAME
    }
}
impl Configurable for Base64Builder {
    /// TODO: add more options to customize the base64 transport.
    fn with_config(self, _conf: &str) -> Result<Self> {
        Ok(self)
    }
}


impl TransportBuilder for Base64Builder {
    fn build(&self, r: &Role) -> Result<TransportInstance> {
        match r {
            Role::Sealer =>   Ok(TransportInstance::new(Box::new(self.sealer()?))),
            Role::Revealer => Ok(TransportInstance::new(Box::new(self.revealer()?))),
        }
    }
}

impl Named for Base64 {
    fn name(&self) -> &'static str {
        NAME
    }
}

impl Default for Base64 {
    fn default() -> Self {
        Self {
            _engine: general_purpose::STANDARD_NO_PAD,
        }
    }
}

impl Base64Builder {
    fn build_seal(&self) -> Result<Box<dyn Seal + Unpin + Send + Sync>> {
        Ok(Box::<Base64>::default())
    }

    fn build_reveal(&self) -> Result<Box<dyn Reveal + Unpin + Send + Sync>> {
        Ok(Box::<Base64>::default())
    }
}

impl WrapTransport for Base64Builder {
    fn sealer(
        &self,
    ) -> Result<Wrapper> {
        let seal = self.build_seal()?;
        let reveal = self.build_reveal()?;
        Ok(Wrapper{seal, reveal})
    }

    fn revealer(
        &self,
    ) -> Result<Wrapper> {
        let seal = self.build_seal()?;
        let reveal = self.build_reveal()?;
        Ok(Wrapper{seal, reveal})
    }
}

impl Seal for Base64 {
    fn seal<'a>(
        &self,
        r: Box<dyn AsyncWrite + Unpin + Send + Sync + 'a>,
    ) -> Box<dyn AsyncWrite + Unpin + Send + Sync + 'a> {
        r
    }
}

impl Reveal for Base64 {
    fn reveal<'a>(
        &self,
        r: Box<dyn AsyncRead + Unpin + Send + Sync + 'a>,
    ) -> Box<dyn AsyncRead + Unpin + Send + Sync + 'a> {
        r
    }
}

// impl Base64Transport {
//     fn new() -> Self {
//         return Base64Transport {};
//     }

//     fn decode<R, W>(reader: &mut R, writer: &mut W) -> Result<u64>
//     where
//         R: Read + ?Sized,
//         W: Write + ?Sized,
//     {
//         let mut buf: Vec<u8> = Vec::with_capacity(1024);
//         let mut enc_buf: Vec<u8> = Vec::with_capacity(1024 * 4 / 3 + 4);

//         let mut total: usize = 0;
//         let mut nw: usize;

//         loop {
//             let nr = match reader.read(&mut enc_buf) {
//                 Ok(n) if n == 0 => break,
//                 Ok(n) => n,
//                 Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
//                 Err(e) => return Err(e),
//             };

//             let ne = general_purpose::STANDARD
//                 .decode_slice(&enc_buf[..nr], &mut buf)
//                 .expect("decode error");

//             nw = writer.write(&mut buf[..ne])?;
//             total += nw;
//             writer.flush();
//         }

//         Ok(total.try_into().unwrap())
//     }

//     fn encode<R, W>(reader: &mut R, writer: &mut W) -> Result<u64>
//     where
//         R: Read + ?Sized,
//         W: Write + ?Sized,
//     {
//         let mut buf: Vec<u8> = Vec::with_capacity(1024);
//         let mut enc_buf: Vec<u8> = Vec::with_capacity(1024 * 4 / 3 + 4);

//         let mut total: usize = 0;
//         let mut nw: usize;

//         loop {
//             let nr = match reader.read(&mut buf) {
//                 Ok(n) if n == 0 => break,
//                 Ok(n) => n,
//                 Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
//                 Err(e) => return Err(e),
//             };

//             let ne = general_purpose::STANDARD
//                 .encode_slice(&buf[..nr], &mut enc_buf)
//                 .expect("encode error");

//             nw = writer.write(&mut buf[..ne])?;
//             total += nw;
//             writer.flush();
//         }

//         Ok(total.try_into().unwrap())
//     }
// }

#[cfg(test)]
mod test {
    use super::*;
    use base64::{Engine as _, engine::general_purpose};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::try_join;

    ///                __              __
    ///                |     (Sealer)    |
    ///         write  | reader [ read ] |===============> echo
    ///                |__             __|                  ||
    ///         __             __                           ||
    ///        |    (Revealer)   |                          ||
    ///        | [ read ] reader | write <===================
    ///        |__             __|
    ///
    #[tokio::test]
    async fn wrap_transport() {
        let wrapper = Base64Builder::default().sealer().unwrap();
        let revealer = wrapper.reveal;
        let sealer = wrapper.seal;
        let (mut client, mut server) = tokio::net::UnixStream::pair().unwrap();

        let server_task = tokio::spawn(async move {
            let (r, w) = server.split();
            let mut wrapped_w = sealer.seal(Box::new(w));
            let mut wrapped_r = revealer.reveal(Box::new(r));
            tokio::io::copy(&mut wrapped_r, &mut wrapped_w)
                .await
                .unwrap();
        });

        let client_task = tokio::spawn(async move {
            let (mut cr, mut cw) = client.split();
            let nw = cw.write(&[0_u8; 1024]).await.unwrap();
            assert_eq!(nw, 1024);

            let mut buf = [0_u8; 1024];
            let nr = cr.read(&mut buf).await.unwrap();
            assert_eq!(nr, 1024);
        });

        try_join!(client_task, server_task).unwrap();
    }


    /// tests showing tha the base64 encode / decode works with tokio::io::AsyncRead / AsyncWrite
    /// traits and don't require the std::io::Read / Write traits.
    #[tokio::test]
    async fn stream_encode_decode() -> std::io::Result<()> {

        let (mut c, mut s) = tokio::io::duplex(128);

        tokio::spawn(async move {
            let mut orig = [0_u8; 128];
            let nr = s.read(&mut orig).await.unwrap();
            let encoded: String = general_purpose::STANDARD_NO_PAD.encode(&orig[..nr]);
            println!("server encoded to: {encoded}");

            _ = s.write(encoded.as_bytes()).await;
        });

        let orig = b"dat!@#@ !@3123 123 1231 1@#2!$(Aa";
        let _ = c.write(orig).await?;

        let mut encoded = [0_u8; 128];
        let nr = c.read(&mut encoded).await?;

        let decoded = general_purpose::STANDARD_NO_PAD.decode(&encoded[..nr]).unwrap();
        assert_eq!(orig.as_slice(), &decoded);

        println!("client decoded to: {}", String::from_utf8(decoded).unwrap());
        Ok(())
    }
}
