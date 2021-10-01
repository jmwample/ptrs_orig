

use std::io;
use std::thread;
use std::time;
use std::env;


const NTHREADS: u32 = 10;

// This is the `main` thread
fn main() {
    // Make a vector to hold the children which are spawned.
    let mut children = vec![];

	if env::var("TOR_PT_EXIT_ON_STDIN_CLOSE") == Ok(String::from("1")) {
		children.push(
			thread::spawn( move || {
				let mut buffer = String::new();
				let stdin = io::stdin(); // We get `Stdin` here.
				while stdin.read_line(&mut buffer).unwrap() != 0 {
					buffer.clear();
				}
			})
		);
	}

    for i in 0..NTHREADS {
        // Spin up another thread
        children.push(thread::spawn(move || {
            println!("this is thread number {}", i);
			thread::sleep(time::Duration::from_secs(3));
        }));
    }

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }
}


// fn main() -> io::Result<()> {

// 	if env::var()
// 	let handle = thread::spawn( move || {
// 		let mut buffer = String::new();
// 		let stdin = io::stdin(); // We get `Stdin` here.
// 		while stdin.read_line(&mut buffer).unwrap() != 0 {
// 			buffer.clear();
// 		}
// 	});

// 	handle.join().unwrap();

// 	Ok(())
// }