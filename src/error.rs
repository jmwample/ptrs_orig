//! Errors that can occur during Pluggable Transport establishment.

use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum PTError {
	#[error("No proxy requested in TOR_PT_PROXY")]
	NoProxyRequested,
	#[error("PROXY-ERROR {0}")]
	ProxyError(String),
    #[error("data store disconnected")]
    Disconnect(#[from] io::Error),
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },
    #[error("unknown data store error")]
    Unknown,
}