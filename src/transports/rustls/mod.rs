use crate::{Error, Result, Stream, Transport};

use futures::Future;
use rustls_pemfile::{certs, pkcs8_private_keys};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::{TlsAcceptor, TlsConnector};
use tracing::trace;

pub(crate) mod certs;

use std::{
    io::{self, BufReader},
    sync::Arc,
};

#[derive(Clone)]
struct Config {
    client_cfg: Option<Arc<rustls::ClientConfig>>,
    server_cfg: Option<Arc<rustls::ServerConfig>>,
}

impl Default for Config {
    fn default() -> Self {
        let common_name = "example.com";
        let subject_alt_names: Vec<String> = vec![
            "example.com".into(),
            "self-signed.example.com".into(),
            "jfaawekmawdvawf.example.com".into(),
        ];
        let cert_set = certs::generate_and_sign(common_name, subject_alt_names)
            .expect("failed to build server certs");

        Self {
            client_cfg: Some(default_client_config_with_root(
                cert_set.ca_pem.as_bytes().to_vec(),
            )),
            server_cfg: Some(default_server_config_with_ca(cert_set).unwrap()),
        }
    }
}

fn default_server_config_with_ca(
    cert_set: certs::SelfSignedSet,
) -> Result<Arc<rustls::ServerConfig>> {
    trace!("cert: {}", cert_set.ca.certificate.serialize_pem().unwrap());
    trace!(
        "key:{}",
        cert_set.ca.certificate.serialize_private_key_pem()
    );

    let mut cert_store = rustls::RootCertStore::empty();

    let ca_cert_reader = &mut BufReader::new(cert_set.ca_pem.as_bytes());
    let ca_cert = rustls::Certificate(certs(ca_cert_reader).unwrap()[0].clone());

    cert_store
        .add(&ca_cert)
        .expect("root CA not added to store");

    let cert_reader = &mut BufReader::new(cert_set.direct.as_bytes());
    let key_reader = &mut BufReader::new(&cert_set.key[..]);

    let cert_chain = certs(cert_reader)
        .unwrap()
        .into_iter()
        .map(rustls::Certificate)
        .collect();
    let mut keys: Vec<rustls::PrivateKey> = pkcs8_private_keys(key_reader)
        .unwrap()
        .into_iter()
        .map(rustls::PrivateKey)
        .collect();

    let server_config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, keys.remove(0))
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

    // Allow using SSLKEYLOGFILE.
    // server_config.key_log = Arc::new(rustls::KeyLogFile::new());

    Ok(Arc::new(server_config))
}

fn default_client_config_with_root(root_cert: Vec<u8>) -> Arc<rustls::ClientConfig> {
    let mut root_store = rustls::RootCertStore::empty();

    let mut root_reader = BufReader::new(&root_cert[..]);

    let roots = match rustls_pemfile::read_one(&mut root_reader)
        .expect("error occured while parsing generated root cert")
        .expect("no root cert provided in generated set")
    {
        rustls_pemfile::Item::X509Certificate(c) => c,
        _ => panic!("bad root cert in cert set provide to client builder"),
    };

    root_store.add(&rustls::Certificate(roots)).unwrap();
    root_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    // Allow using SSLKEYLOGFILE.
    // config.key_log = Arc::new(rustls::KeyLogFile::new());

    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Arc::new(config)
}

// impl TransportBuilder for RustlsBuilder {
// 	fn build(&self, r: &Role) -> Result<crate::TransportInstance> {
// 		match r {
// 			Role::Sealer => Ok(TransportInstance::new(Box::new(Client::from_config(self.config.as_ref())))),
// 			// Role::Revealer => Ok(TransportInstance::new(Box::new(Client::from_config(self.config.as_ref())))),
// 			Role::Revealer => Err(Error::Other("not implemented yet".into())),
// 		}
// 	}
// }

struct Client {
    config: Config,
}

impl Client {
    fn _from_config(c: Option<&Config>) -> Self {
        let config = match c {
            Some(config) => config.clone(),
            None => Config::default(),
        };
        Client { config }
    }

    async fn wrap<'a, A>(&self, a: A) -> Result<Box<dyn Stream + 'a>>
    where
        A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
    {
        let config = self
            .config
            .client_cfg
            .clone()
            .ok_or(Error::Other("no client config provided".into()))?;
        let connector = TlsConnector::from(config).clone();
        let server_name = "www.rust-lang.org".try_into().unwrap();
        let stream = connector.connect(server_name, a).await?;
        Ok(Box::new(stream))
    }
}

impl<'a, A> Transport<'a, A> for Client
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn wrap(&self, a: A) -> impl Future<Output = Result<Box<dyn Stream + 'a>>> {
        self.wrap(a)
    }
}

struct Server {
    config: Config,
}

// #[async_trait]
impl<'a, A> Transport<'a, A> for Server
where
    A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
    fn wrap(&self, a: A) -> impl Future<Output = Result<Box<dyn Stream + 'a>>> {
        self.wrap(a)
    }
}

impl Server {
    async fn wrap<'a, A>(&self, a: A) -> Result<Box<dyn Stream + 'a>>
    where
        A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
    {
        let config = self
            .config
            .server_cfg
            .clone()
            .ok_or(Error::Other("no server config provided".into()))?;
        let acceptor = TlsAcceptor::from(config);
        let stream = acceptor.accept(a).await?;
        Ok(Box::new(stream))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Result;

    #[tokio::test]
    async fn async_tls_rustls_read_write() -> Result<()> {
        let (mut c, mut s) = tokio::io::duplex(128);
        let message = b"hello world asldkasda daweFAe0342323;l3 />?123";
        let config = Config::default();
        let server_config = config.clone();

        tokio::spawn(async move {
            let server = Server {
                config: server_config,
            };
            let wrapped_server_conn = server.wrap(&mut s).await.unwrap();

            let (mut reader, mut writer) = tokio::io::split(wrapped_server_conn);
            let n = tokio::io::copy(&mut reader, &mut writer).await.unwrap();
            assert_eq!(n, message.len() as u64);
            // writer.flush().await.unwrap();
        });

        let client = Client { config };
        let _wrapped_client_conn = client.wrap(&mut c).await?;

        // tls.write_all(
        // 	concat!(
        // 		"GET / HTTP/1.1\r\n",
        // 		"Host: www.rust-lang.org\r\n",
        // 		"Connection: close\r\n",
        // 		"Accept-Encoding: identity\r\n",
        // 		"\r\n"
        // 	)
        // 	.as_bytes(),
        // )
        // .unwrap();

        // let ciphersuite = tls
        // 	.conn
        // 	.negotiated_cipher_suite()
        // 	.unwrap();

        // writeln!(
        // 	&mut std::io::stderr(),
        // 	"Current ciphersuite: {:?}",
        // 	ciphersuite.suite()
        // )
        // .unwrap();

        // let mut plaintext = Vec::new();

        // tls.read_to_end(&mut plaintext).unwrap();

        // stdout().write_all(&plaintext).unwrap();

        Ok(())
    }
}
