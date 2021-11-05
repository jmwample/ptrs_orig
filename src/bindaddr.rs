//! maybe put under PT module?

use std::env;
use std::net::SocketAddr;

use crate::args::*;

use crate::error::PTError;
use crate::hashmap;
use crate::pt::resolve_addr;

const TOR_PT_SERVER_BINDADDR: &str = "TOR_PT_SERVER_BINDADDR";
const TOR_PT_SERVER_TRANSPORTS: &str = "TOR_PT_SERVER_TRANSPORTS";
const TOR_PT_SERVER_TRANSPORT_OPTIONS: &str = "TOR_PT_SERVER_TRANSPORT_OPTIONS";

#[derive(Clone, PartialEq, Debug)]
struct BindAddr {
	pub method_name: String,
	pub addr: SocketAddr,

	/// Options from TOR_PT_SERVER_TRANSPORT_OPTIONS relevant to this transport.
	pub options: Args,
}

fn get_server_bindaddrs() -> Result<Vec<BindAddr>, PTError> {
	let mut addrs: Vec<BindAddr> = vec![];

	// Parse the list of server transport options.
	let server_transport_options = env::var(TOR_PT_SERVER_TRANSPORT_OPTIONS)?;
	let options_map = parse_server_transport_options(&server_transport_options)?;

	// Get the list of all requested bindaddrs.
	let server_bindaddr = env::var(TOR_PT_SERVER_BINDADDR)?;

	let mut seen = vec![];
	for spec in server_bindaddr.split(',') {
		let parts: Vec<&str> = spec.split('-').collect();
		if parts.len() != 2 {
			return Err(PTError::ParseError(format!(
				"TOR_PT_SERVER_BINDADDR: {}: doesn't contain \"-\"",
				spec,
			)));
		}

		// Check for duplicate method names: "Applications MUST NOT set
		// more than one <address>:<port> pair per PT name."
		let method_name = parts[0];
		if seen.contains(&method_name) {
			return Err(PTError::ParseError(format!(
				"TOR_PT_SERVER_BINDADDR: {}: duplicate method name {}",
				spec, method_name,
			)));
		}
		seen.push(method_name);

		let addr = resolve_addr(parts[1])?;
		let options = options_map
			.get(method_name)
			.unwrap_or(&Args::new())
			.to_owned();

		let bindaddr = BindAddr {
			method_name: method_name.to_string(),
			addr,
			options,
		};
		addrs.push(bindaddr);
	}

	// Filter by TOR_PT_SERVER_TRANSPORTS.
	let server_transports_env = env::var(TOR_PT_SERVER_TRANSPORTS)?;
	let server_transports: Vec<&str> = server_transports_env.split(',').collect();
	let result = filter_bindaddrs(addrs, &server_transports);

	Ok(result)
}

fn filter_bindaddrs(bindaddrs: Vec<BindAddr>, method_names: &[&str]) -> Vec<BindAddr> {
	let mut result: Vec<BindAddr> = vec![];
	for addr in bindaddrs.iter() {
		if method_names.contains(&addr.method_name.as_str()) {
			result.push(addr.to_owned());
		}
	}
	result
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_get_server_bindaddrs_good() {
		let empty_bindaddr_vec: Vec<BindAddr> = vec![];
		let test_cases = vec![
			(
				"alpha-1.2.3.4:1111",
				"alpha,beta,gamma",
				"",
				vec![BindAddr {
					method_name: String::from("alpha"),
					addr: "1.2.3.4:1111".parse().unwrap(),
					options: Args::new(),
				}],
			),
			("alpha-1.2.3.4:1111", "xxx", "", empty_bindaddr_vec.clone()),
			// In the past, "*" meant to return all known transport names.
			// But now it has no special meaning.
			// https://bugs.torproject.org/15612
			(
				"alpha-1.2.3.4:1111,beta-[1:2::3:4]:2222",
				"*",
				"",
				empty_bindaddr_vec.clone(),
			),
			(
				"alpha-1.2.3.4:1111,beta-[1:2::3:4]:2222",
				"alpha,beta,gamma",
				"alpha:k1=v1;beta:k2=v2;gamma:k3=v3",
				vec![
					BindAddr {
						method_name: String::from("alpha"),
						addr: "1.2.3.4:1111".parse().unwrap(),
						options: from_str_map(hashmap!("k1" => vec!["v1"])),
					},
					BindAddr {
						method_name: String::from("beta"),
						addr: "[1:2::3:4]:2222".parse().unwrap(),
						options: from_str_map(hashmap!("k2" => vec!["v2"])),
					},
				],
			),
			(
				"alpha-1.2.3.4:1111,beta-[1:2::3:4]:2222",
				"alpha,beta,gamma",
				"alpha:k1=v1,beta:k2=v2,gamma:k3=v3",
				vec![
					BindAddr {
						method_name: String::from("alpha"),
						addr: "1.2.3.4:1111".parse().unwrap(),
						options: from_str_map(hashmap!("k1" => vec!["v1,beta:k2=v2,gamma:k3=v3"])),
					},
					BindAddr {
						method_name: String::from("beta"),
						addr: "[1:2::3:4]:2222".parse().unwrap(),
						options: Args::new(),
					},
				],
			),
			(
				"trebuchet-127.0.0.1:1984,ballista-127.0.0.1:4891",
				"trebuchet,ballista",
				"trebuchet:secret=nou;trebuchet:cache=/tmp/cache;ballista:secret=yes",
				vec![
					BindAddr {
						method_name: String::from("trebuchet"),
						addr: "127.0.0.1:1984".parse().unwrap(),
						options: from_str_map(
							hashmap!("secret" => vec!["nou"], "cache" => vec!["/tmp/cache"]),
						),
					},
					BindAddr {
						method_name: String::from("ballista"),
						addr: "127.0.0.1:4891".parse().unwrap(),
						options: from_str_map(hashmap!("secret" => vec!["yes"])),
					},
				],
			),
		];

		env::remove_var(TOR_PT_SERVER_BINDADDR);
		env::remove_var(TOR_PT_SERVER_TRANSPORTS);
		env::remove_var(TOR_PT_SERVER_TRANSPORT_OPTIONS);
		for (bind_addr, server_transports, server_transport_options, expected) in test_cases {
			env::set_var(TOR_PT_SERVER_BINDADDR, &bind_addr);
			env::set_var(TOR_PT_SERVER_TRANSPORTS, &server_transports);
			env::set_var(TOR_PT_SERVER_TRANSPORT_OPTIONS, &server_transport_options);

			match get_server_bindaddrs() {
				Ok(bindaddrs) => {
					assert_eq!(bindaddrs, expected, "TOR_PT_SERVER_BINDADDR={} TOR_PT_SERVER_TRANSPORTS={} TOR_PT_SERVER_TRANSPORT_OPTIONS={} → {:?} (expected {:?})",
					bind_addr, server_transports, server_transport_options, bindaddrs, expected);
				}

				Err(err) => {
					panic!("TOR_PT_SERVER_BINDADDR={} TOR_PT_SERVER_TRANSPORTS={} TOR_PT_SERVER_TRANSPORT_OPTIONS={} unexpectedly returned an error: {}",
					bind_addr, server_transports, server_transport_options, err);
				}
			}
		}
	}

	#[test]
	fn test_get_server_bindaddrs_bad() {
		let test_cases = vec![
			// bad TOR_PT_SERVER_BINDADDR
			("alpha", "alpha", ""),
			("alpha-1.2.3.4", "alpha", ""),
			// missing TOR_PT_SERVER_TRANSPORTS
			("alpha-1.2.3.4:1111", "", "alpha:key=value"),
			// bad TOR_PT_SERVER_TRANSPORT_OPTIONS
			("alpha-1.2.3.4:1111", "alpha", "key=value"),
			// no escaping is defined for TOR_PT_SERVER_TRANSPORTS or
			// TOR_PT_SERVER_BINDADDR.
			(r"alpha\,beta-1.2.3.4:1111", r"alpha\,beta", ""),
			// duplicates in TOR_PT_SERVER_BINDADDR
			// https://bugs.torproject.org/21261
			(r"alpha-0.0.0.0:1234,alpha-[::]:1234", r"alpha", ""),
			(r"alpha-0.0.0.0:1234,alpha-0.0.0.0:1234", r"alpha", ""),
		];

		env::remove_var(TOR_PT_SERVER_BINDADDR);
		env::remove_var(TOR_PT_SERVER_TRANSPORTS);
		env::remove_var(TOR_PT_SERVER_TRANSPORT_OPTIONS);

		for (bind_addr, server_transports, server_transport_options) in test_cases {
			env::set_var(TOR_PT_SERVER_BINDADDR, &bind_addr);
			match server_transports {
				"" => env::remove_var(TOR_PT_SERVER_TRANSPORTS),
				_ => env::set_var(TOR_PT_SERVER_TRANSPORTS, &server_transports),
			};
			env::set_var(TOR_PT_SERVER_TRANSPORT_OPTIONS, &server_transport_options);

			match get_server_bindaddrs() {
				Ok(_) => {
					panic!("TOR_PT_SERVER_BINDADDR={} TOR_PT_SERVER_TRANSPORTS={} TOR_PT_SERVER_TRANSPORT_OPTIONS={} unexpectedly succeeded",
				bind_addr, server_transports, server_transport_options);
				}

				Err(_) => {}
			}
		}
	}

	#[test]
	fn test_filter_bindaddrs() {
		let expected = vec![BindAddr {
			method_name: String::from("alpha"),
			addr: "1.2.3.4:1111".parse().unwrap(),
			options: from_str_map(hashmap!("k1" => vec!["v1"])),
		}];
		let bindaddrs = vec![
			BindAddr {
				method_name: String::from("alpha"),
				addr: "1.2.3.4:1111".parse().unwrap(),
				options: from_str_map(hashmap!("k1" => vec!["v1"])),
			},
			BindAddr {
				method_name: String::from("beta"),
				addr: "[1:2::3:4]:2222".parse().unwrap(),
				options: from_str_map(hashmap!("k2" => vec!["v2"])),
			},
		];
		let filter_list = ["alpha", "gamma"];

		let result = filter_bindaddrs(bindaddrs, &filter_list);
		assert_eq!(result, expected);
	}
}
