use std::{fmt::Display, str::FromStr};

use hex::FromHexError;
use rcgen::RcgenError;

pub type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {}
#[derive(Debug)]
pub enum Error {
    Other(Box<dyn std::error::Error>),
    IOError(std::io::Error),
    EncodeError(Box<dyn std::error::Error>),
    CertGenError(RcgenError),
    NullTransport,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Error::Other(e) => write!(f, "{}", e),
            Error::IOError(e) => write!(f, "{}", e),
            Error::EncodeError(e) => write!(f, "{}", e),
            Error::CertGenError(e) => write!(f, "{}", e),
            Error::NullTransport => write!(f, "NullTransport"),
        }
    }
}

unsafe impl Send for Error {}

impl Error {
    pub fn new<T: Into<Box<dyn std::error::Error>>>(e: T) -> Self {
        Error::Other(e.into())
    }
}

impl FromStr for Error {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Error::new(s))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e)
    }
}

impl From<Box<std::io::Error>> for Error {
    fn from(e: Box<std::io::Error>) -> Self {
        Error::IOError(*e)
    }
}

impl From<FromHexError> for Error {
    fn from(e: FromHexError) -> Self {
        Error::EncodeError(Box::new(e))
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        Error::Other(e)
    }
}

impl From<RcgenError> for Error {
    fn from(e: RcgenError) -> Self {
        Error::CertGenError(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_other_error() {
        let err = Error::new("some other error");
        assert_eq!(format!("{}", err), "some other error");
    }

    #[test]
    fn test_display_io_error() {
        let err = Error::IOError(std::io::Error::new(
            std::io::ErrorKind::Other,
            "some io error",
        ));
        assert_eq!(format!("{}", err), "some io error");
    }

    #[test]
    fn test_display_encode_error() {
        let err = Error::EncodeError(Box::new(FromHexError::InvalidHexCharacter {
            c: 'z',
            index: 0,
        }));
        assert_eq!(format!("{}", err), "Invalid character 'z' at position 0");
    }

    #[test]
    fn test_display_null_transport_error() {
        let err = Error::NullTransport;
        assert_eq!(format!("{}", err), "NullTransport");
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "some io error");
        let err = Error::from(io_err);
        assert_eq!(format!("{}", err), "some io error");
    }

    #[test]
    fn test_from_encode_error() {
        let hex_err = FromHexError::InvalidHexCharacter { c: 'z', index: 0 };
        let err = Error::from(hex_err);
        assert_eq!(format!("{}", err), "Invalid character 'z' at position 0");
    }

    #[test]
    fn test_from_other_error() {
        let other_err = Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "some other error",
        ));
        let err = Error::from(other_err);
        assert_eq!(format!("{}", err), "some other error");
    }
}
