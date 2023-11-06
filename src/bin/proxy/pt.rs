// use crate::handler::Split;
// use ptrs::{
//     transports::{AsTransport, Transport, Transports},
//     Result,
// };

// use tokio::io::{AsyncRead, AsyncWrite};

// pub fn get_transport(name: &str) -> Result<Box<dyn Transport>> {
//     Transports::from_str(name)?.as_transport()
// }

// pub fn wrap_with<I, O>(name: &str, args: Vec<String>, stream: I) -> Result<O>
// where
//     I: Split + AsyncRead + AsyncWrite + Unpin + Send + 'static,
//     O: Split + AsyncRead + AsyncWrite + Unpin + Send + 'static,
// {
//     Transports::from_str(name)?
//         .as_transport()?
//         .with_config(args)?
//         .wrap(stream)
//         .await
// }
