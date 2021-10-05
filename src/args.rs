//! Key–value mappings for the representation of client and server options.

use itertools::Itertools;
use std::collections::HashMap;

use crate::error::PTError;
use crate::hashmap;

/// Arguments maintained as a map of string keys to a list of values.
/// It is similar to url.Values.
// #[derive(Debug, Clone, PartialEq)]
// pub struct Args(HashMap<String, Vec<String>>);
pub type Args = HashMap<String, Vec<String>>;

pub trait Track {
	fn add(&mut self, key: &str, value: &str);
	fn retrieve(&self, key: &str) -> Option<String>;
}

impl Track for Args {
	fn add(&mut self, key: &str, value: &str) {
		// value either exists or is allocated here.
		self.entry(key.to_string()).or_insert(vec![]);

		// therefor value should never be None and it is safe to unwrap.
		self.get_mut(key).unwrap().push(value.to_string());
	}

	fn retrieve(&self, key: &str) -> Option<String> {
		match self.get(key) {
			Some(v) => match v.len() {
				0 => None,
				_ => Some(v[0].to_owned()),
			},
			None => None,
		}
	}
}

/// Encode a name–value mapping so that it is suitable to go in the ARGS option
/// of an SMETHOD line. The output is sorted by key. The "ARGS:" prefix is not
/// added.
///
/// "Equal signs and commas [and backslashes] MUST be escaped with a backslash."
fn encode_smethod_args(maybe_args: Option<&Args>) -> String {
	if maybe_args.is_none() {
		return String::from("");
	}

	let escape = |s: &str| -> String { backslash_escape(s, vec!['=', ',']) };

	maybe_args
		.unwrap()
		.iter()
		.sorted()
		.map(|(key, values)| {
			values
				.iter()
				.map(|value| format!("{}={}", escape(key), escape(value)))
				.collect::<Vec<String>>()
				.join(",")
		})
		.collect::<Vec<String>>()
		.join(",")
}

fn backslash_escape(s: &str, set: Vec<char>) -> String {
	let mut result = String::new();
	s.chars().for_each(|a| {
		if a == '\\' || set.contains(&a) {
			result.push('\\');
		}
		result.push(a);
	});
	result
}

/// Return the index of the next unescaped byte in s that is in the term set, or
/// else the length of the string if no terminators appear. Additionally return
/// the unescaped string up to the returned index.
fn index_unescaped<'a>(s: &'a str, term: Vec<char>) -> Result<(usize, String), PTError> {
	let mut unesc = String::new();
	let mut i: usize = 0;
	while i < s.len() {
		let mut c = s.chars().nth(i).unwrap();

		// is c a terminator character?
		if term.contains(&c) {
			break;
		}
		if c == '\\' {
			i += 1;
			if i >= s.len() {
				return Err(PTError::ParseError(format!(
					"nothing following final escape in \"{}\"",
					s
				)));
			}
			c = s.chars().nth(i).unwrap();
		}
		unesc.push(c);
		i += 1;
	}
	Ok((i, unesc))
}

/// Parse a name–value mapping as from an encoded SOCKS username/password.
///
/// "First the '<Key>=<Value>' formatted arguments MUST be escaped, such that all
/// backslash, equal sign, and semicolon characters are escaped with a
/// backslash."
fn parse_client_parameters(params: &str) -> Result<Args, PTError> {
	let mut args = Args::new();
	if params.is_empty() {
		return Ok(args);
	}

	let mut i: usize = 0;
	loop {
		let begin = i;

		// Read the key.
		let (offset, key) = index_unescaped(&params[i..], vec!['=', ';'])?;

		i += offset;
		// End of string or no equals sign?
		if i >= params.len() || params.chars().nth(i).unwrap() != '=' {
			return Err(PTError::ParseError(format!(
				"parsing client params found no equals sign in {}",
				&params[begin..i]
			)));
		}

		// Skip the equals sign.
		i += 1;

		// Read the value.
		let (offset, value) = index_unescaped(&params[i..], vec![';'])?;

		i += offset;
		if key.len() == 0 {
			return Err(PTError::ParseError(format!(
				"parsing client params encountered empty key in {}",
				&params[begin..i]
			)));
		}
		args.add(&key, &value);

		if i >= params.len() {
			break;
		}

		// Skip the semicolon.
		i += 1;
	}

	Ok(args)
}

/// transport name to value mapping as from TOR_PT_SERVER_TRANSPORT_OPTIONS
pub type Opts = HashMap<String, Args>;

/// Parse a transport–name–value mapping as from TOR_PT_SERVER_TRANSPORT_OPTIONS.
///
/// "...a semicolon-separated list of <key>:<value> pairs, where <key> is a PT
/// name and <value> is a k=v string value with options that are to be passed to
/// the transport. Colons, semicolons, equal signs and backslashes must be
/// escaped with a backslash."
///
/// Example: scramblesuit:key=banana;automata:rule=110;automata:depth=3
fn parse_server_transport_options(s: &str) -> Result<Opts, PTError> {
	let mut opts = Opts::new();
	if s.len() == 0 {
		return Ok(opts);
	}
	let mut i: usize = 0;
	loop {
		let begin = i;
		// Read the method name.
		let (offset, method_name) = index_unescaped(&s[i..], vec![':', '=', ';'])?;

		i += offset;
		// End of string or no colon?
		if i >= s.len() || s.chars().nth(i).unwrap() != ':' {
			return Err(PTError::ParseError(format!("no colon in {}", &s[begin..i])));
		}
		// Skip the colon.
		i += 1;

		// Read the key.
		let (offset, key) = index_unescaped(&s[i..], vec!['=', ';'])?;

		i += offset;
		// End of string or no equals sign?
		if i >= s.len() || s.chars().nth(i).unwrap() != '=' {
			return Err(PTError::ParseError(format!(
				"no equals sign in {}",
				&s[begin..i]
			)));
		}
		// Skip the equals sign.
		i += 1;

		// Read the value.
		let (offset, value) = index_unescaped(&s[i..], vec![';'])?;

		i += offset;
		if method_name.len() == 0 {
			return Err(PTError::ParseError(format!(
				"empty method name in {}",
				&s[begin..i]
			)));
		}
		if key.len() == 0 {
			return Err(PTError::ParseError(format!(
				"empty key in {}",
				&s[begin..i]
			)));
		}

		opts.entry(method_name)
			.and_modify(|e| e.add(&key, &value))
			.or_insert(hashmap! {key => vec![value]});

		if i >= s.len() {
			break;
		}
		// Skip the semicolon.
		i += 1;
	}
	Ok(opts)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::hashmap;

	#[test]
	fn test_get_args() {
		let args: Args = hashmap!(
			String::from("a") => vec![],
			String::from("b") => vec![String::from("value")],
			String::from("c") => vec![String::from("v1"), String::from("v2"), String::from("v3")]
		);

		let empty: Args = HashMap::new();

		if let Some(v) = empty.retrieve("a") {
			panic!("unexpected result from `get` on empty Args: {:?}", v);
		}

		if let Some(v) = args.retrieve("a") {
			panic!("unexpected get success for \"{}\" → {:?}", "a", v);
		}

		match args.retrieve("b") {
			Some(v) => assert_eq!(v, "value", "Get({}) → {:?} (expected {})", "b", v, "value"),
			None => panic!("Unexpected Get failure for \"{}\"", "b"),
		}

		match args.retrieve("c") {
			Some(v) => assert_eq!(v, "v1", "Get({}) → {:?} (expected {})", "c", v, "v1"),
			None => panic!("Unexpected Get failure for \"{}\"", "c"),
		}

		if let Some(v) = args.retrieve("d") {
			panic!("unexpected get success for \"{}\" → {:?}", "d", v);
		}
	}

	#[test]
	fn test_add_args() {
		let mut args = Args::new();
		let mut expected: Args = HashMap::new();
		assert_eq!(args, expected, "{:?} != {:?}", args, expected);

		args.add("k1", "v1");
		expected = hashmap!(
			String::from("k1")=>vec![String::from("v1")]
		);
		assert_eq!(args, expected, "{:?} != {:?}", args, expected);

		args.add("k2", "v2");
		expected = hashmap!(
			String::from("k1")=>vec![String::from("v1")],
			String::from("k2") => vec![String::from("v2")]
		);
		assert_eq!(args, expected, "{:?} != {:?}", args, expected);

		args.add("k1", "v3");
		expected = hashmap!(
			String::from("k1") => vec![String::from("v1"), String::from("v3")],
			String::from("k2") => vec![String::from("v2")]
		);
		assert_eq!(args, expected, "{:?} != {:?}", args, expected);
	}

	#[test]
	fn test_parse_client_parameters() {
		let bad_cases = vec![
			"key",
			"key\\",
			"=value",
			"==value",
			"==key=value",
			"key=value\\",
			"a=b;key=value\\",
			"a;b=c",
			";",
			"key=value;",
			";key=value",
			"key\\=value",
		];
		let good_cases = vec![
			("", HashMap::new()),
			("key=", hashmap!("key" => vec![""])),
			("key==", hashmap!("key" => vec!["="])),
			("key=value", hashmap!("key" => vec!["value"])),
			("a=b=c", hashmap!("a" => vec!["b=c"])),
			("a=bc==", hashmap!("a" => vec!["bc=="])),
			("a\\=b=c", hashmap!("a=b" => vec!["c"])),
			("key=a\nb", hashmap!("key" => vec!["a\nb"])),
			("key=value\\;", hashmap!("key" => vec!["value;"])),
			("key=\"value\"", hashmap!("key" => vec!["\"value\""])),
			("\"key=value\"", hashmap!("\"key" => vec!["value\""])),
			(
				"key=\"\"value\"\"",
				hashmap!("key" => vec!["\"\"value\"\""]),
			),
			(
				"key=value;key=value",
				hashmap!("key" => vec!["value", "value"]),
			),
			(
				"key=value1;key=value2",
				hashmap!("key" => vec!["value1", "value2"]),
			),
			(
				"key1=value1;key2=value2;key1=value3",
				hashmap!("key1" => vec!["value1", "value3"], "key2" => vec!["value2"]),
			),
			(
				"\\;=\\;;\\\\=\\;",
				hashmap!(";" => vec![";"], "\\" => vec![";"]),
			),
			(
				"shared-secret=rahasia;secrets-file=/tmp/blob",
				hashmap!("shared-secret" => vec!["rahasia"], "secrets-file" => vec!["/tmp/blob"]),
			),
			(
				"rocks=20;height=5.6",
				hashmap!("rocks" => vec!["20"], "height" => vec!["5.6"]),
			),
		];

		for input in bad_cases {
			match parse_client_parameters(input) {
				Ok(_) => panic!("{} unexpectedly succeeded", input),
				Err(_) => {}

				// TODO: Validate error types
				// Err(_) => { todo!("Validate error types")}
				// Err(err) => assert_eq!(err, Box::new(PTError::Unknown)),
			}
		}

		for (input, exected_map) in good_cases.iter() {
			// Convert all &str to String to keep tests readable
			let expected: Args = exected_map
				.iter()
				.map(|(k, vs)| (k.to_string(), vs.iter().map(|v| v.to_string()).collect()))
				.collect();

			match parse_client_parameters(input) {
				Ok(args) => assert_eq!(
					args, expected,
					"{} → {:?} (expected {:?})",
					input, args, expected
				),
				Err(err) => panic!("{} unexpectedly returned an error: {}", input, err),
			}
		}
	}

	#[test]
	fn parse_good_server_transport_options() {
		let good_cases = [
			("", hashmap! {}),
			(
				"t:k=v",
				hashmap! {
					"t" => hashmap!{"k" => vec!["v"]}
				},
			),
			(
				"t:k=v=v",
				hashmap! {
					"t" => hashmap!{"k" => vec!["v=v"]},
				},
			),
			(
				"t:k=vv==",
				hashmap! {
					"t" => hashmap!{"k" => vec!["vv=="]},
				},
			),
			(
				"t1:k=v1;t2:k=v2;t1:k=v3",
				hashmap! {
					"t1" => hashmap!{"k" => vec!["v1", "v3"]},
					"t2" => hashmap!{"k" => vec!["v2"]},
				},
			),
			(
				"t\\:1:k=v;t\\=2:k=v;t\\;3:k=v;t\\\\4:k=v",
				hashmap! {
					"t:1" =>  hashmap!{"k" => vec!["v"]},
					"t=2" =>  hashmap!{"k" => vec!["v"]},
					"t;3" =>  hashmap!{"k" => vec!["v"]},
					"t\\4" => hashmap!{"k" => vec!["v"]},
				},
			),
			(
				"t:k\\:1=v;t:k\\=2=v;t:k\\;3=v;t:k\\\\4=v",
				hashmap! {
					"t" => hashmap!{
						"k:1" =>  vec!["v"],
						"k=2" =>  vec!["v"],
						"k;3" =>  vec!["v"],
						"k\\4" => vec!["v"],
					},
				},
			),
			(
				"t:k=v\\:1;t:k=v\\=2;t:k=v\\;3;t:k=v\\\\4",
				hashmap! {
					"t" => hashmap!{"k" => vec!["v:1", "v=2", "v;3", "v\\4"]},
				},
			),
			(
				"trebuchet:secret=nou;trebuchet:cache=/tmp/cache;ballista:secret=yes",
				hashmap! {
					"trebuchet" => hashmap!{
						"secret" => vec!["nou"],
						"cache" => vec!["/tmp/cache"]
					},
					"ballista" =>  hashmap!{"secret" => vec!["yes"]},
				},
			),
		];
		for (input, expected) in good_cases {
			match parse_server_transport_options(input) {
				Ok(opts) => {
					// Convert all &str to String to keep tests readable
					let expected_opts: Opts = expected
						.iter()
						.map(|(opt_key, args)| {
							(
								opt_key.to_string(),
								args.iter()
									.map(|(k, vs)| {
										(k.to_string(), vs.iter().map(|v| v.to_string()).collect())
									})
									.collect(),
							)
						})
						.collect();
					assert_eq!(
						opts, expected_opts,
						"{} → {:?} (expected {:?})",
						input, opts, expected
					)
				}
				Err(err) => panic!("{:?} unexpectedly returned error {}", input, err),
			}
		}
	}

	#[test]
	fn parse_bad_server_transport_options() {
		let bad_cases = [
			"t\\",
			":=",
			"t:=",
			":k=",
			":=v",
			"t:=v",
			"t:=v",
			"t:k\\",
			"t:k=v;",
			"abc",
			"t:",
			"key=value",
			"=value",
			"t:k=v\\",
			"t1:k=v;t2:k=v\\",
			"t:=key=value",
			"t:==key=value",
			"t:;key=value",
			"t:key\\=value",
		];

		for input in bad_cases {
			match parse_server_transport_options(input) {
				Ok(_) => panic!("{} unexpectedly succeeded", input),
				Err(_) => {}

				// TODO: Validate error types
				// Err(_) => {todo!("Validate error types")}
				// Err(err) => assert_eq!(err, Box::new(PTError::Unknown)),
			}
		}
	}

	#[test]
	fn test_encode_smethod_args() {
		let tests = [
			// (None, ""),
			(HashMap::new(), ""),
			(
				hashmap! {"j"=>vec!["v1", "v2", "v3"], "k"=>vec!["v1", "v2", "v3"]},
				"j=v1,j=v2,j=v3,k=v1,k=v2,k=v3",
			),
			(
				hashmap! {"=,\\"=>vec!["=", ",", "\\"]},
				"\\=\\,\\\\=\\=,\\=\\,\\\\=\\,,\\=\\,\\\\=\\\\",
			),
			(hashmap! {"secret"=>vec!["yes"]}, "secret=yes"),
			(
				hashmap! {"secret"=> vec!["nou"], "cache" => vec!["/tmp/cache"]},
				"cache=/tmp/cache,secret=nou",
			),
		];

		assert_eq!("", encode_smethod_args(None));

		for (input_map, expected) in tests.iter() {
			let input: Args = input_map
				.iter()
				.map(|(k, vs)| (k.to_string(), vs.iter().map(|v| v.to_string()).collect()))
				.collect();

			let encoded = encode_smethod_args(Some(&input));
			assert_eq!(
				&encoded, expected,
				"{:?} → {} (expected {})",
				input, encoded, expected
			)
		}
	}
}
