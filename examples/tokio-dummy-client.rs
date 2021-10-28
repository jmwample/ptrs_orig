// Dummy no-op pluggable transport client. Works only as a managed proxy.
//
// Usage (in torrc):
// 	UseBridges 1
// 	Bridge dummy X.X.X.X:YYYY
// 	ClientTransportPlugin dummy exec dummy-client
//
// Because this transport doesn't do anything to the traffic, you can use the
// ORPort of any ordinary bridge (or relay that has DirPort set) in the bridge
// line; it doesn't have to declare support for the dummy transport.

extern crate ptrs;

use std::env;
use std::io;
use std::net::TcpListener;
use std::process::exit;
use std::thread;
use std::time;

use ptrs::pt;
use tokio;
use tokio::signal;
use tokio::sync::{mpsc, watch};

/*
#[tokio::main]
async fn main() -> Result<_,_> {
	let pt_info = pt::client_setup().unwrap();

	if pt_info.proxy_url != None {
		println!("proxy not supported");
		std::process::exit(1);
	}

	// Closed when all references are dropped.
	let mut listeners: Vec<TcpListener> = vec![];

	for method_name in pt_info.method_names {
		match method_name.as_ref() {
			"dummy" => {
				let ln = TcpListener::bind("127.0.0.1:80").unwrap();
				// TODO: Allocate socks listener and run the accept
				// thread for handling connections
				listeners.push(ln);
			}
			_ => {
				println!("CMETHOD-ERROR {} {}", method_name, "no such method");
			}
		}
	}
	println!("{} {}", "CMETHODS", "DONE");
	// Handle Ctrl-D if TOR_PT_EXIT_ON_STDIN_CLOSE
	let handle = if env::var("TOR_PT_EXIT_ON_STDIN_CLOSE") == Ok(String::from("1")) {
		// This environment variable means we should treat EOF on stdin
		// just like SIGTERM: https://bugs.torproject.org/15435
		thread::spawn(move || {
			let mut buffer = String::new();
			let stdin = io::stdin();

			while stdin.read_line(&mut buffer).unwrap() != 0 {
				buffer.clear();
			}
			exit(0);
		})
	} else {
		//If unset empty thread will just exit.
		thread::spawn(move || {})
	};

	handle.join().unwrap();
}
*/

async fn say_world() {
	thread::sleep(time::Duration::from_secs(3));
	println!("world");
}

fn shutdown() {
	println!("shutting down");
}

#[tokio::main]
async fn main() {
	let (shutdown_send, mut shutdown_recv) = mpsc::unbounded_channel::<()>();

	// ... spawn application as separate task ...
	//
	// application uses shutdown_send in case a shutdown was issued from inside
	// the application
	println!("hello");

	tokio::select! {
		_ = say_world() => {},
		_ = signal::ctrl_c() => {},
		_ = shutdown_recv.recv() => {},
	};

	// send shutdown signal to application and wait
	shutdown()
}

// resources:
// https://tokio.rs/tokio/tutorial
// https://tokio.rs/tokio/topics/shutdown
// https://docs.rs/tokio/1.12.0/tokio/signal/fn.ctrl_c.html

// https://github.com/tokio-rs/mini-redis
// https://github.com/tokio-rs/mini-redis/blob/master/src/bin/server.rs
// https://github.com/tokio-rs/mini-redis/blob/master/src/server.rs
