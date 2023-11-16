pub mod stream;
use crate::Named;

// mod trait_alias;
// pub use trait_alias::*;

// mod trait_with_fn_generics;
// pub use trait_with_fn_generics::*;

// use std::io::{self, Read, Result, Write};

pub struct PlainTransport {}

impl Named for PlainTransport {
    fn name(&self) -> String {
        "identity".into()
    }
}

pub fn default() -> PlainTransport {
    PlainTransport {}
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::common::pipes; // {pipe_set, pipes}

//     use std::thread;

//     #[test]
//     fn e2e_plain() -> Result<()> {
//         // imagine that the client did TcpStream::connect and got `client`, and simultaneously the
//         // server side called accept and got `remote`.
//         let (mut client, mut remote) = pipes()?;

//         let mut buf: &[u8] = b"hello world";

//         let client_transport = default();
//         let (client_encode_f, client_decode_f) = client_transport.dial_handler(&mut client)?;

//         thread::spawn(move || {
//             let server_transport = default();
//             let (server_encode_f, server_decode_f) =
//                 server_transport.listen_handler(&mut remote).unwrap();

//             let mut server_out = vec![0_u8; 1024];
//             let server_read_result = server_decode_f(&mut remote, &mut server_out);
//             assert!(server_read_result.is_ok());
//             let snr = server_read_result.unwrap() as usize;
//             assert_eq!(snr as usize, buf.len());
//             assert_eq!(
//                 std::str::from_utf8(&server_out[..snr]).unwrap(),
//                 "hello world"
//             );

//             //echo the message back to the client
//             let server_write_result = server_encode_f(&mut &server_out[..snr], &mut remote);
//             assert!(server_write_result.is_ok());
//             let snw = server_write_result.unwrap() as usize;
//             assert_eq!(snw as usize, buf.len());
//         });

//         let cnw = client_encode_f(&mut buf, &mut client)?;
//         assert_eq!(cnw as usize, buf.len());

//         let mut client_out = vec![0_u8; 1024];
//         let cnr = client_decode_f(&mut client, &mut client_out)?;
//         assert_eq!(cnr as usize, buf.len());

//         assert_eq!(
//             std::str::from_utf8(&client_out[..cnr as usize]).unwrap(),
//             "hello world"
//         );

//         Ok(())
//     }
// }
