//! Pluggable Transports

use std::env;
use url::Url;

use crate::error::PTError;

const TOR_PT_PROXY: &str = "TOR_PT_PROXY";
const TOR_PT_MANAGED_TRANSPORT_VER: &str = "TOR_PT_MANAGED_TRANSPORT_VER";
const TOR_PT_CLIENT_TRANSPORTS: &str = "TOR_PT_CLIENT_TRANSPORTS";

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

/// Get a pluggable transports version offered by Tor and understood by us, if
/// any. The only version we understand is "1". This function reads the
/// environment variable TOR_PT_MANAGED_TRANSPORT_VER.
fn get_managed_transport_version<'a>() -> Result<String, PTError> {
	const TRANSPORT_VERSION: &str = "1";
	for s in env::var(TOR_PT_MANAGED_TRANSPORT_VER)?.split("") {
		if s == TRANSPORT_VERSION {
			return Ok(s.to_string());
		}
	}

	Err(PTError::VersionError(String::from("no-version")))
}

/// Get the list of method names requested by Tor. This function reads the
/// environment variable TOR_PT_CLIENT_TRANSPORTS.
fn get_client_transports<'a>() -> Result<Vec<String>, PTError> {
	// TOR_PT_CLIENT_TRANSPORTS
	Ok(env::var(TOR_PT_CLIENT_TRANSPORTS)?
		.split(",")
		.map(|s| s.to_string())
		.collect::<Vec<String>>())
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

/// Returns true iff keyword contains only bytes allowed in a PT→Tor output line
/// keyword.
/// <KeywordChar> ::= <any US-ASCII alphanumeric, dash, and underscore>
fn keyword_is_safe(keyword: &str) -> bool {
	for b in keyword.chars() {
		match b {
			b if '0' <= b && b <= '9' => continue,
			b if 'A' <= b && b <= 'Z' => continue,
			b if 'a' <= b && b <= 'z' => continue,
			'-' => continue,
			'_' => continue,
			_ => return false,
		}
	}
	true
}

/// Returns true iff arg contains only bytes allowed in a PT→Tor output line arg.
/// <ArgChar> ::= <any US-ASCII character but NUL or NL>
fn arg_is_safe(arg: &str) -> bool {
	for b in arg.chars() {
		match b as u8 {
			b if b >= 0x80 => return false,
			0x00 | 0x0a => return false,
			_ => continue,
		}
	}
	true
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_resolve_addr() {
		todo!()
	}

	#[test]
	fn test_get_server_bindaddrs() {
		todo!()
	}

	#[test]
	fn test_read_auth_cookie() {
		todo!()
	}

	#[test]
	fn test_compute_server_hash() {
		todo!()
	}

	#[test]
	fn test_compute_client_hash() {
		todo!()
	}

	#[test]
	fn test_ext_or_send_command() {
		todo!()
	}

	#[test]
	fn test_ext_or_send_user_addr() {
		todo!()
	}

	#[test]
	fn test_ext_or_port_send_transport() {
		todo!()
	}

	#[test]
	fn test_ext_or_port_send_done() {
		todo!()
	}

	#[test]
	fn test_ext_or_port_recv_command() {
		todo!()
	}

	#[test]
	fn test_ext_or_port_set_metadata() {
		todo!()
	}

	#[test]
	fn test_ext_or_port_setup_fail_set_deadline() {
		todo!()
	}

	#[test]
	fn test_make_state_dir() {
		todo!()
	}

	#[test]
	fn test_encode_cstring() {
		todo!()
	}

	#[test]
	fn test_keyword_is_safe() {
		let tests = [
			(r"", true),
			(
				r"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_",
				true,
			),
			(r"CMETHOD", true),
			(r"CMETHOD:", false),
			(r"a b c", false),
			(r"CMETHOD\x7f", false),
			(r"CMETHOD\x80", false),
			(r"CMETHOD\x81", false),
			(r"CMETHOD\xff", false),
			(r"\xffCMETHOD", false),
			(r"CMÉTHOD", false),
		];

		for (input, expected) in tests {
			let is_safe = keyword_is_safe(input);
			assert_eq!(
				is_safe, expected,
				"keyword_is_safe(\"{}\") → {} (expected {})",
				input, is_safe, expected
			)
		}
	}

	#[test]
	fn test_arg_is_safe() {
		let tests = [
			("", true),
			("abc", true),
			("127.0.0.1:8000", true),
			("étude", false),
			("a\nb", false),
			("a\\b", true),
			("ab\\", true),
			("ab\\\n", false),
			("ab\n\\", false),
			("abc\x7f", true),
			// // Rust doesn't allow char hex escape over 0x7f
			// ("abc\x80", false),
			// ("abc\x81", false),
			// ("abc\xff", false),
			("abc\u{0080}", false),
			("abc\u{0081}", false),
			("abc\u{00ff}", false),
			("var=GVsbG8\\=", true),
		];

		for (input, expected) in tests {
			let is_safe = arg_is_safe(input);
			assert_eq!(
				is_safe, expected,
				"arg_is_safe(\"{}\") → {} (expected {})",
				input, is_safe, expected
			)
		}
	}

	#[test]
	fn test_get_managed_transport_ver() {
		let good_cases = [("1", "1"), ("1,1", "1"), ("1,2", "1"), ("2,1", "1")];
		let bad_cases = ["", "2"];
		env::remove_var(TOR_PT_MANAGED_TRANSPORT_VER);
		match get_managed_transport_version() {
			Ok(ver) => panic!("empty environment unexpectedly returned {:?}", ver),
			Err(err) => {
				assert_eq!(
					err,
					PTError::from(env::VarError::NotPresent),
					"empty environment returned unexpected error: {}",
					err
				);
			}
		}

		for (input, expected) in good_cases {
			env::set_var(TOR_PT_MANAGED_TRANSPORT_VER, input);
			match get_managed_transport_version() {
				Ok(ver) => {
					assert_eq!(
						ver, expected,
						"TOR_PT_MANAGED_TRANSPORT_VER={} → {} (expected {})",
						input, ver, expected
					);
				}
				Err(err) => {
					panic!(
						"TOR_PT_MANAGED_TRANSPORT_VER={} unexpectedly returned an error: {}",
						input, err
					);
				}
			}
		}

		for input in bad_cases {
			env::set_var(TOR_PT_MANAGED_TRANSPORT_VER, input);
			match get_managed_transport_version() {
				Ok(ver) => {
					panic!(
						"TOR_PT_MANAGED_TRANSPORT_VER={} unexpectedly succeeded and returned {:?}",
						input, ver
					);
				}
				Err(err) => {
					let expected = PTError::VersionError(String::from("no-version"));
					assert_eq!(
						err, expected,
						"TOR_PT_MANAGED_TRANSPORT_VER={} returned error \"{}\" expected \"{}\"",
						input, err, expected
					)
				}
			}
		}
	}

	#[test]
	fn test_get_client_transports() {
		let test_cases = [
			("alpha", vec!["alpha"]),
			("alpha,beta", vec!["alpha", "beta"]),
			("alpha,beta,gamma", vec!["alpha", "beta", "gamma"]),
			// In the past, "*" meant to return all known transport names.
			// But now it has no special meaning.
			// https://bugs.torproject.org/15612
			("*", vec!["*"]),
			// No escaping is defined for TOR_PT_CLIENT_TRANSPORTS.
			(r"alpha\,beta", vec![r"alpha\", "beta"]),
		];

		env::remove_var(TOR_PT_CLIENT_TRANSPORTS);
		match get_client_transports() {
			Ok(ver) => panic!("empty environment unexpectedly returned {:?}", ver),
			Err(err) => {
				assert_eq!(
					err,
					PTError::from(env::VarError::NotPresent),
					"empty environment returned unexpected error: {}",
					err
				);
			}
		}

		for (input, expected) in test_cases {
			env::set_var(TOR_PT_CLIENT_TRANSPORTS, input);
			match get_client_transports() {
				Ok(transports) => {
					assert_eq!(
						transports, expected,
						"TOR_PT_CLIENT_TRANSPORTS={} → {:?} (expected {:?})",
						input, transports, expected
					);
				}
				Err(err) => {
					panic!(
						"TOR_PT_CLIENT_TRANSPORTS={} unexpectedly returned an error: {}",
						input, err
					);
				}
			}
		}
	}

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
