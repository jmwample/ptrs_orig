// use std::io::{self, Read, Result, Write};

use crate::sync::constructions::stream::StreamHandler;
use crate::Result;
use crate::{
    // transports::{AsTransport, Transport},
    Configurable,
    Named,
};

use hex::{decode_to_slice, encode_to_slice, encode_upper};

use std::io::{BufWriter, Error, ErrorKind, Read, Write};
use std::str::FromStr;

pub const NAME: &str = "hex";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Case {
    Upper,
    Lower,
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub case: Case,
}

impl FromStr for Config {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "upper" => Ok(Config { case: Case::Upper }),
            "lower" => Ok(Config { case: Case::Lower }),
            _ => Err(Error::new(
                ErrorKind::Other,
                format!("Bad config, unknown case: {}", s),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HexEncoder {
    config: Config,
}

impl Configurable for HexEncoder {
    fn with_config(self, args: &str) -> Result<Self> {
        Ok(HexEncoder {
            config: Config::from_str(args)?,
        })
    }
}

impl HexEncoder {
    pub fn new() -> Self {
        HexEncoder {
            config: Config { case: Case::Upper },
        }
    }

    pub fn stream_encode_fn() -> Result<Box<dyn crate::sync::constructions::stream::StreamHandler>>
    {
        // let _h = Self::new();
        crate::sync::constructions::stream::from_transform(|r, mut w| {
            // Ok(h.encode(r, w)?)
            let mut buf = [0_u8; 1024];
            let nr = r.read(&mut buf)?;
            Ok(w.write(&buf[..nr])?)
        })
    }

    pub fn encode<T: AsRef<[u8]>>(&self, data: T, out: &mut [u8]) -> Result<usize> {
        let l: usize;

        match self.config.case {
            Case::Upper => {
                encode_to_slice(data.as_ref(), out)
                    .map_err(|e| Error::new(ErrorKind::Other, format!("encode error: {e}")))?;
                l = out.len()
            }
            Case::Lower => {
                let s = encode_upper(data.as_ref());
                l = s.len();
                _ = BufWriter::new(out).write(s.as_bytes())?;
            }
        }
        Ok(l)
    }

    pub fn decode<T: AsRef<[u8]>>(&self, data: T, out: &mut [u8]) -> Result<()> {
        let l = data.as_ref().len() / 2;
        if out.len() < l {
            return Err(Error::new(
                ErrorKind::Other,
                format!("output buffer too small: {} < {}", out.len(), l),
            )
            .into());
        }

        decode_to_slice(data.as_ref(), &mut out[..l])
            .map_err(|e| Error::new(ErrorKind::Other, format!("decode error: {e}")))?;
        Ok(())
    }
}

impl Default for HexEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HexEncoder> for Box<dyn StreamHandler> {
    fn from(h: HexEncoder) -> Self {
        let _h = h;
        Box::new(move |r: &mut dyn Read, w: &mut dyn Write| -> Result<u64> {
            let mut buf = [0_u8; 1024];
            let mut out = [0_u8; 1024];
            let mut total = 0_u64;
            loop {
                let nr = r.read(&mut buf)?;
                if nr == 0 {
                    break;
                }
                let nw = _h.encode(&buf[..nr], &mut out)?;
                w.write_all(&out[..nw])?;
                total += nw as u64;
            }
            Ok(total)
        })
    }
}

impl Named for HexEncoder {
    fn name(&self) -> &'static str {
        "hex"
    }
}

impl Named for &HexEncoder {
    fn name(&self) -> &'static str {
        NAME
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encode_decode() -> Result<()> {
        let message = b"hello world";
        let mut encoded = [0_u8; 1024];

        let h = HexEncoder::new().with_config("lower")?;
        let n = h.encode(message, &mut encoded).expect("failed to encode");

        let mut decoded = [0_u8; 1024];
        h.decode(&encoded[..n], &mut decoded)
            .expect("failed to decode");

        assert_eq!(message, &decoded[..message.len()]);

        Ok(())
    }
}
