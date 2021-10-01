//! Pluggable Transports

use std::env;
use url::Url;

use crate::error::PTError;

const TOR_PT_PROXY: &str = "TOR_PT_PROXY";

fn emit(keyword: &str, v: &[&str]) {
	let mut vv = String::new();
	v.iter().for_each(|a| {
		vv.push(' ');
		vv.push_str(a);
	});

	println!("{}{}", keyword, vv);
}

// This structure is returned by [`client_setup`]. It consists of a list of method
// names and the upstream proxy URL, if any.
#[derive(Debug, Clone)]
pub struct ClientInfo {
	pub method_names: Vec<String>,
	pub proxy_url: Option<Url>,
}

/// Check the client pluggable transports environment, emitting an error message
/// and returning a non-nil error if any error is encountered. Returns a
/// ClientInfo struct.
///
/// If your program needs to know whether to call [`client_setup`] or [`server_setup`]
/// (i.e., if the same program can be run as either a client or a server), check
/// whether the `TOR_PT_CLIENT_TRANSPORTS` environment variable is set:
///
/// ```
/// 	if std::env::var("TOR_PT_CLIENT_TRANSPORTS").is_ok() {
/// 		// Client mode; call pt::client_setup.
/// 	} else {
/// 		// Server mode; call pt::server_setup.
/// 	}
/// ```
pub fn client_setup() -> Result<ClientInfo, Box<dyn std::error::Error>> {
	let ver = get_managed_transport_version()?;
	emit("VERSION", &[&ver]);

	Ok(ClientInfo {
		proxy_url: get_proxy_url()?,
		method_names: get_client_transports()?,
	})
}

fn get_managed_transport_version() -> Result<String, Box<dyn std::error::Error>> {
	Ok(String::from(""))
}

fn get_client_transports() -> Result<Vec<String>, Box<dyn std::error::Error>> {
	Ok(vec![String::from("")])
}

/// Get the upstream proxy URL. Returns [`PTError::NoProxyRequested`] if no proxy is requested. The
/// function ensures that the Scheme and Host fields are set; i.e., that the URL
/// is absolute. It additionally checks that the Host field contains both a host
/// and a port part. This function reads the environment variable TOR_PT_PROXY.
///
/// This function doesn't check that the scheme is one of Tor's supported proxy
/// schemes; that is, one of "http", "socks5", or "socks4a". The caller must be
/// able to handle any returned scheme (which may be by calling ProxyError if
/// it doesn't know how to handle the scheme).
fn get_proxy_url() -> Result<Option<Url>, Box<dyn std::error::Error>> {
	let tor_pt_proxy = match env::var(TOR_PT_PROXY) {
		Ok(url) => url,
		Err(err) => match err {
			env::VarError::NotPresent => return Ok(None),
			_ => return Err(Box::new(err)),
		},
	};

	let url = Url::parse(&tor_pt_proxy)?;

	if url.scheme() == "" {
		return Err(Box::new(PTError::ProxyError(String::from(
			"missing scheme",
		))));
	}

	if !url.has_authority() {
		return Err(Box::new(PTError::ProxyError(String::from(
			"missing authority",
		))));
	}

	if !url.has_host() {
		return Err(Box::new(PTError::ProxyError(String::from("missing host"))));
	}

	if url.port().is_none() {
		return Err(Box::new(PTError::ProxyError(String::from("missing port"))));
	}

	Ok(Some(url))
}

pub struct ServerInfo {}

pub fn server_setup() -> Result<ServerInfo, Box<dyn std::error::Error>> {
	Ok(ServerInfo {})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn proxy_url_parse() {
		let bad_cases: Vec<&str> = vec![
			"bogus",
			"http:",
			"://127.0.0.1",
			"//127.0.0.1",
			"http:127.0.0.1",
			"://[::1]",
			"//[::1]",
			"http:[::1]",
			"://localhost",
			"//localhost",
			"http:localhost",
			// No port in these.
			"http://127.0.0.1",
			"socks4a://127.0.0.1",
			"socks5://127.0.0.1",
			"http://127.0.0.1:",
			"http://[::1]",
			"http://localhost",
			"unknown://localhost/whatever",
			// No host in these.
			"http://:8080",
			"socks4a://:1080",
			"socks5://:1080",
		];

		// (input, expected)
		let good_cases: Vec<(&str, &str)> = vec![
			("http://127.0.0.1:8080", "http://127.0.0.1:8080/"),
			("http://127.0.0.1:8080/", "http://127.0.0.1:8080/"),
			("http://127.0.0.1:8080/path", "http://127.0.0.1:8080/path"),
			("http://[::1]:8080", "http://[::1]:8080/"),
			("http://[::1]:8080/", "http://[::1]:8080/"),
			("http://[::1]:8080/path", "http://[::1]:8080/path"),
			("http://localhost:8080", "http://localhost:8080/"),
			("http://localhost:8080/", "http://localhost:8080/"),
			("http://localhost:8080/path", "http://localhost:8080/path"),
			("http://user@localhost:8080", "http://user@localhost:8080/"),
			(
				"http://user:password@localhost:8080",
				"http://user:password@localhost:8080/",
			),
			("socks5://localhost:1080", "socks5://localhost:1080"),
			("socks4a://localhost:1080", "socks4a://localhost:1080"),
			(
				"unknown://localhost:9999/whatever",
				"unknown://localhost:9999/whatever",
			),
		];

		env::remove_var(TOR_PT_PROXY);
		match get_proxy_url() {
			Ok(url) => match url {
				Some(u) => panic!("empty environment returned {:?}", u),
				None => {}
			},
			Err(err) => {
				panic!("empty environment returned unexpected error: {}", err);
			}
		}

		for (input, expected) in good_cases {
			env::set_var(TOR_PT_PROXY, input);
			match get_proxy_url() {
				Ok(u) => {
					let url = String::from(u.unwrap());
					assert_eq!(
						&url, expected,
						"TOR_PT_PROXY={} → {} (expected {})",
						input, &url, expected
					);
				}
				Err(err) => {
					panic!(
						"TOR_PT_PROXY={} unexpectedly returned an error: {}",
						input, err
					);
				}
			}
		}

		for input in bad_cases {
			env::set_var(TOR_PT_PROXY, input);
			match get_proxy_url() {
				Ok(url) => {
					panic!(
						"TOR_PT_PROXY={} unexpectedly succeeded and returned {:?}",
						input, url
					);
				}
				Err(_) => {}
			}
		}
	}
}
