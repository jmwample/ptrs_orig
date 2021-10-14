
# Pluggable Transports in Rust (PTRS)

[![codecov](https://codecov.io/gh/jmwample/ptrs/branch/main/graph/badge.svg?token=M5366KWEA4)](https://codecov.io/gh/jmwample/ptrs)

PTRS is a library for writing Tor pluggable transports in Rust.

* [Pluggable Transport Specification (Version 1)](https://gitweb.torproject.org/torspec.git/tree/pt-spec.txt)

* [Extended ORPort and TransportControlPort](https://gitweb.torproject.org/torspec.git/tree/proposals/196-transport-control-ports.txt)

* [Tor Extended ORPort Authentication](https://gitweb.torproject.org/torspec.git/tree/proposals/217-ext-orport-auth.txt)


See the included example programs for examples of how to use the
library. To build them, enter their directory and run "go build".

* examples/dummy-client.rs

* examples/dummy-server.rs

The recommended way to start writing a new transport plugin is to copy
dummy-client or dummy-server and make changes to it.

There is browseable documentation here:
[TODO](#)

To the extent possible under law, the authors have dedicated all
copyright and related and neighboring rights to this software to the
public domain worldwide. This software is distributed without any
warranty. See COPYING.

## FFI

The library exposes a set of C ABI bindings, those are defined in the `ptrs_ffi.h`
header file. The C bindings can be used with C/C++, Swift (using a bridging
header) or C# (using
[DLLImport](https://docs.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.dllimportattribute?view=netcore-2.2)
with
[CallingConvention](https://docs.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.dllimportattribute.callingconvention?view=netcore-2.2)
set to `Cdecl`).
