//! Errors that can occur during Pluggable Transport establishment.

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PTError {
	///
	#[error("ENV-ERROR {0}")]
	EnvError(#[from] ::std::env::VarError),

	///
	#[error("PARSE-ERROR {0}")]
	ParseError(String),

	///
	#[error("PROXY-ERROR {0}")]
	ProxyError(String),

	///
	#[error("VERSION-ERROR {0}")]
	VersionError(String),

	///
	#[error("CMETHOD-ERROR {0} {1}")]
	CMethodError(String, String),

	///
	#[error("SMETHOD-ERROR {0} {1}")]
	SMethodError(String, String),

	/// unexpected error occurred.
	#[error("UNKNOWN-ERROR occurred")]
	Unknown,
	// /// No Proxy was requested by the client
	// #[error("No proxy requested in TOR_PT_PROXY")]
	// NoProxyRequested,
}

#[test]
fn error_format() {
	let e = PTError::EnvError(std::env::VarError::NotPresent);
	assert_eq!(e.to_string(), "ENV-ERROR environment variable not found");

	let e = PTError::VersionError("XYZ".to_string());
	assert_eq!(e.to_string(), "VERSION-ERROR XYZ");

	let e = PTError::ParseError("XYZ".to_string());
	assert_eq!(e.to_string(), "PARSE-ERROR XYZ");

	let e = PTError::ProxyError("XYZ".to_string());
	assert_eq!(e.to_string(), "PROXY-ERROR XYZ");

	let e = PTError::SMethodError("method".to_string(), "XYZ".to_string());
	assert_eq!(e.to_string(), "SMETHOD-ERROR method XYZ");

	let e = PTError::CMethodError("method".to_string(), "XYZ".to_string());
	assert_eq!(e.to_string(), "CMETHOD-ERROR method XYZ");

	let e = PTError::Unknown;
	assert_eq!(e.to_string(), "UNKNOWN-ERROR occurred")
}
