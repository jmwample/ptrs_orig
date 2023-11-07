use socks5_proto::{Address, Error, Reply};
use socks5_server::{connection::state::NeedAuthenticate, Command, IncomingConnection};
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpStream,
};

pub async fn _handle(conn: IncomingConnection<(), NeedAuthenticate>) -> Result<(), Error> {
    let conn = match conn.authenticate().await {
        Ok((conn, _)) => conn,
        Err((err, mut conn)) => {
            let _ = conn.shutdown().await;
            return Err(err);
        }
    };

    match conn.wait().await {
        Ok(Command::Associate(associate, _)) => {
            let replied = associate
                .reply(Reply::CommandNotSupported, Address::unspecified())
                .await;

            let mut conn = match replied {
                Ok(conn) => conn,
                Err((err, mut conn)) => {
                    let _ = conn.shutdown().await;
                    return Err(Error::Io(err));
                }
            };

            let _ = conn.close().await;
        }
        Ok(Command::Bind(bind, _)) => {
            let replied = bind
                .reply(Reply::CommandNotSupported, Address::unspecified())
                .await;

            let mut conn = match replied {
                Ok(conn) => conn,
                Err((err, mut conn)) => {
                    let _ = conn.shutdown().await;
                    return Err(Error::Io(err));
                }
            };

            let _ = conn.close().await;
        }
        Ok(Command::Connect(connect, addr)) => {
            let target = match addr {
                Address::DomainAddress(domain, port) => {
                    let domain = String::from_utf8_lossy(&domain);
                    TcpStream::connect((domain.as_ref(), port)).await
                }
                Address::SocketAddress(addr) => TcpStream::connect(addr).await,
            };

            if let Ok(mut target) = target {
                let replied = connect
                    .reply(Reply::Succeeded, Address::unspecified())
                    .await;

                let mut conn = match replied {
                    Ok(conn) => conn,
                    Err((err, mut conn)) => {
                        let _ = conn.shutdown().await;
                        return Err(Error::Io(err));
                    }
                };

                let res = io::copy_bidirectional(&mut target, &mut conn).await;
                let _ = conn.shutdown().await;
                let _ = target.shutdown().await;

                res?;
            } else {
                let replied = connect
                    .reply(Reply::HostUnreachable, Address::unspecified())
                    .await;

                let mut conn = match replied {
                    Ok(conn) => conn,
                    Err((err, mut conn)) => {
                        let _ = conn.shutdown().await;
                        return Err(Error::Io(err));
                    }
                };

                let _ = conn.shutdown().await;
            }
        }
        Err((err, mut conn)) => {
            let _ = conn.shutdown().await;
            return Err(err);
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Result;

    use socks5_server::{auth::NoAuth, Server};

    use once_cell::sync::OnceCell;
    use std::sync::{Arc, Mutex};
    use tokio::{
        io::{copy, split, AsyncReadExt, AsyncWriteExt},
        net::{TcpListener, TcpStream},
        runtime::Runtime,
    };
    use tokio_socks::tcp::Socks5Stream;

    const ECHO_SERVER_ADDR: &str = "127.0.0.1:10007";
    const SOCKS_SERVER_ADDR: &str = "127.0.0.1:10018";
    const MSG: &[u8] = b"hello";

    pub async fn echo_server() -> Result<()> {
        let listener = TcpListener::bind(ECHO_SERVER_ADDR).await?;
        // println!("echo server listening");
        loop {
            let (mut stream, _) = listener.accept().await?;
            tokio::spawn(async move {
                let (mut reader, mut writer) = stream.split();
                // println!("echo-  ing");
                copy(&mut reader, &mut writer).await.unwrap();
            });
        }
    }

    // ================================================================================

    #[tokio::test]
    async fn bind_with_socket_no_auth() -> Result<()> {
        // println!("HERE: test_fn 0");

        let listener = TcpListener::bind(SOCKS_SERVER_ADDR).await.unwrap();
        let auth = Arc::new(NoAuth) as Arc<_>;

        let server = Server::new(listener, auth);

        runtime().lock().unwrap().spawn(async move {
            while let Ok((conn, _)) = server.accept().await {
                tokio::spawn(async move {
                    match _handle(conn).await {
                        Ok(()) => {}
                        Err(err) => eprintln!("{err}"),
                    }
                });
            }
        });

        test_connect().await
    }

    pub async fn test_connect() -> Result<()> {
        let stream = Socks5Stream::connect(SOCKS_SERVER_ADDR, ECHO_SERVER_ADDR).await?;

        tokio::spawn(async move {
            // println!("HERE: test_fn_runtime 0");
            tokio::spawn(async move {
                let (mut reader, mut writer) = split(stream);
                copy(&mut reader, &mut writer).await.unwrap();
            });
        });

        let socket = TcpStream::connect(SOCKS_SERVER_ADDR).await?;
        // println!("HERE: test_bind 0");

        socket.set_nodelay(true)?;
        let mut conn = Socks5Stream::connect_with_socket(socket, ECHO_SERVER_ADDR).await?;
        // conn.write_all(b"GET /\n\n").await?;

        // println!("HERE: test_bind 1");
        conn.write_all(MSG).await?;
        let mut buf = [0; 5];
        conn.read_exact(&mut buf[..]).await?;
        assert_eq!(&buf[..], MSG);
        Ok(())
    }

    pub fn runtime() -> &'static Mutex<Runtime> {
        static RUNTIME: OnceCell<Mutex<Runtime>> = OnceCell::new();
        RUNTIME.get_or_init(|| {
            let runtime = Runtime::new().expect("Unable to create runtime");
            runtime.spawn(async { echo_server().await.expect("Unable to bind") });
            Mutex::new(runtime)
        })
    }
}
