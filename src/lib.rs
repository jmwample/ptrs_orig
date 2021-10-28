//! # Pluggable Transports in Rust (PTRS)
//!
//! PTRS is a library for writing Tor pluggable transports in Rust.
//!
//! * [Pluggable Transport Specification (Version 1)](https://gitweb.torproject.org/torspec.git/tree/pt-spec.txt)
//! * [Extended ORPort and TransportControlPort](https://gitweb.torproject.org/torspec.git/tree/proposals/196-transport-control-ports.txt)
//! * [Tor Extended ORPort Authentication](https://gitweb.torproject.org/torspec.git/tree/proposals/217-ext-orport-auth.txt)
//!
//! See the included example programs for examples of how to use the
//! library. To build them, enter their directory and run "go build".
//!
//! - examples/dummy-client.rs
//! - examples/dummy-server.rs
//!
//! The recommended way to start writing a new transport plugin is to copy
//! dummy-client or dummy-server and make changes to it.
//!
//! There is browseable documentation here: [TODO](#)
//!
//! To the extent possible under law, the authors have dedicated all
//! copyright and related and neighboring rights to this software to the
//! public domain worldwide. This software is distributed without any
//! warranty. See COPYING.
//!
//! ### Future Developments:
//! - JNI support & examples
//! - WASM support & example
//! - golang shim & examples
//! - python clib & examples
//!

// uncomment when API and first draft is done
// #![warn(missing_debug_implementations, rust_2018_idioms, missing_docs)]

// More expressive io errors
#![feature(io_error_more)]

pub mod args;
pub mod error;
pub mod pt;
mod socks;

pub mod ffi;

#[macro_export(local_inner_macros)]
/// Create a **HashMap** from a list of key-value pairs
///
/// ## Example
///
/// ```ignore
/// # #[macro_use] extern crate ptrs;
/// # fn main() {
///
/// let map = hashmap!{
///     "a" => 1,
///     "b" => 2,
/// };
/// assert_eq!(map["a"], 1);
/// assert_eq!(map["b"], 2);
/// assert_eq!(map.get("c"), None);
/// # }
/// ```
macro_rules! hashmap {
    ($($key:expr => $value:expr,)+) => { hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = crate::count!(@count $($key),*);
            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! count {
	// Private patters used for counting
	//
	// This is a way to force the type as slice of ()
	// turns array into slice then calls slice implementation of len
	//
	// This is evaluated at compile time so there are no more allocations
    // to run the macro. Unit () is a zero size type.
	//
	// (<[()]>)::len(...)    - treat this as a Unit slice and the take the length
    //
    // The (@single ...) target pattern allows us to substitute Units for whatever
	// object is in the expr so we can count with a const object that doesn't
	// require an allocation. See
	// [The Little Book of Rust Macros](https://veykril.github.io/tlborm/decl-macros/building-blocks/counting.html)
	// for more detail.
	(@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(count!(@single $rest)),*]));
}

/// hashmap macro should not allow mixed types to compile
/// ```compile_fail
/// let x: Vec<u32> = crate::hashmap![42 => 42, 43 =>"foo"];
/// ``
#[allow(dead_code)]
struct CompileFailTest;

/// hashmap macro should not allow for just a comma to be valid syntax
/// ```compile_fail
/// let x: Vec<u32> = crate::hashmap![,];
/// ```
#[allow(dead_code)]
struct CompileFailCommaOnlyTest;

#[cfg(test)]
mod macro_tests {
    use super::*;
    use ::std::collections::HashMap;

    #[test]
    fn single() {
        let x: HashMap<u32, u32> = hashmap! {42=> 42};
        assert!(!x.is_empty());
        assert_eq!(x.len(), 1);
        assert_eq!(x[&42], 42);
        assert_eq!(x.get(&43), None);
    }

    #[test]
    fn double() {
        let x: HashMap<u32, u32> = hashmap![42 => 42, 43 => 43];
        assert!(!x.is_empty());
        assert_eq!(x.len(), 2);
        assert_eq!(x[&42], 42);
        assert_eq!(x[&43], 43);
    }

    #[test]
    fn clone_2_non_literal() {
        let mut y = Some(42);
        let x: HashMap<u32, u32> = hashmap![y.take().unwrap() => 42, 43 =>43];
        assert!(!x.is_empty());
        assert_eq!(x.len(), 2);
        assert_eq!(x[&42], 42);
        assert_eq!(x[&43], 43);
    }

    #[test]
    fn empty_hashmap() {
        let x: HashMap<u32, u32> = hashmap! {};
        assert!(x.is_empty());
    }

    #[test]
    fn trailing() {
        let x: HashMap<&'static str, u32> = hashmap! {
            "asdasdfasdfasdfasdfasdfasdcasdcasdcaksdcas" => 0,
            "asdasdfasdfasdfasdfasdfasdcasdcasdcaksdcas" => 1,
            "asdasdfasdfasdfasdfasdfasdcasdcasdcaksdcas" => 2,
            "asdasdfasdfasdfasdfasdfasdcasdcasdcaksdcas" => 3
        };
        assert!(!x.is_empty());
    }

    #[test]
    fn test_hashmap() {
        let names = hashmap! {
            1 => "one",
            2 => "two",
        };
        assert_eq!(names.len(), 2);
        assert_eq!(names[&1], "one");
        assert_eq!(names[&2], "two");
        assert_eq!(names.get(&3), None);

        let empty: HashMap<i32, i32> = hashmap! {};
        assert_eq!(empty.len(), 0);

        let _nested_compiles = hashmap! {
            1 => hashmap!{0 => 1 + 2,},
            2 => hashmap!{1 => 1,},
        };
    }
}
