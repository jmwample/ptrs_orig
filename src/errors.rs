use std::str::FromStr;

use hex::FromHexError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Other(Box<dyn std::error::Error>),
    IOError(std::io::Error),
    EncodeError(Box<dyn std::error::Error>),
    NullTransport,
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
