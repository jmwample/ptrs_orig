/// C bindings for the PTRS Pluggable Transports library

// use std::ffi::{CStr, CString};

#[allow(non_camel_case_types)]
#[repr(C)]
/// Indicates the operation required from the caller
pub enum result_type {
	/// No operation is required.
	WIREGUARD_DONE = 0,
	/// Write dst buffer to network. Size indicates the number of bytes to write.
	WRITE_TO_NETWORK = 1,
	/// Some error occurred, no operation is required. Size indicates error code.
	WIREGUARD_ERROR = 2,
	/// Write dst buffer to the interface as an ipv4 packet. Size indicates the number of bytes to write.
	WRITE_TO_TUNNEL_IPV4 = 4,
	/// Write dst buffer to the interface as an ipv6 packet. Size indicates the number of bytes to write.
	WRITE_TO_TUNNEL_IPV6 = 6,
}
