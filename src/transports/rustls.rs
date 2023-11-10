
use crate::{Transport, Result, Role, Error, TransportBuilder, TransportInstance};

use async_trait::async_trait;
use rcgen::generate_simple_self_signed;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::{TlsConnector, TlsAcceptor};
use rustls_pemfile::{certs, rsa_private_keys};
use tracing::trace;

use std::{
	sync::Arc,
	io::{self, Cursor},
};

#[derive(Clone)]
struct Config {
	client_cfg: Option<Arc<rustls::ClientConfig>>,
	server_cfg: Option<Arc<rustls::ServerConfig>>
}

impl Default for Config {
	fn default() -> Self {

		Self {
			client_cfg: Some(default_client_config()),
			server_cfg: Some(default_server_config().unwrap()),
		}
	}
}

fn default_server_config() -> Result<Arc<rustls::ServerConfig>> {

	// let certs = load_certs(&options.cert)?;
    // let key = load_keys(&options.key)?;
	let (certs, key) = gen_self_signed_cert()?;

	let server_config = rustls::ServerConfig::builder()
		.with_safe_defaults()
		.with_no_client_auth()
		.with_single_cert(certs, key)
		.map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

	// Allow using SSLKEYLOGFILE.
	// server_config.key_log = Arc::new(rustls::KeyLogFile::new());

	Ok(Arc::new(server_config))
}

fn gen_self_signed_cert() -> Result<(Vec<rustls::Certificate>, rustls::PrivateKey)> {
	// Generate a certificate that's valid for "localhost" and "hello.world.example"
	let subject_alt_names = vec!["hello.world.example".to_string(), "localhost".to_string()];
	let cert = generate_simple_self_signed(subject_alt_names)?;

	// Get rustls compatible cert
	let mut cert_r = Cursor::new(cert.serialize_pem()?);
	let certs = rustls_pemfile::certs(&mut cert_r)?;
	let certificate = certs.into_iter().map(rustls::Certificate).collect();

	// Get rust compatible private key
	let mut privkey_r = Cursor::new(cert.serialize_private_key_pem());
	let mut keys = rustls_pemfile::pkcs8_private_keys(&mut privkey_r)?;
	let key = match keys.len() {
		0 => Err(format!("No PKCS8-encoded private key found"))?,
		1 => rustls::PrivateKey(keys.remove(0)),
		_ => Err(format!("More than one PKCS8-encoded private key found"))?,
	};


	trace!("cert: {}", cert.serialize_pem().unwrap());
	trace!("key:{}", cert.serialize_private_key_pem());

	Ok((vec![certificate], key))
}

fn default_client_config() -> Arc<rustls::ClientConfig> {
	let mut root_store = rustls::RootCertStore::empty();
	root_store.add_trust_anchors(
		webpki_roots::TLS_SERVER_ROOTS
			.iter()
			.map(|ta| {
				rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
					ta.subject,
					ta.spki,
					ta.name_constraints,
				)
			})
	);

	// Allow using SSLKEYLOGFILE.
	// config.key_log = Arc::new(rustls::KeyLogFile::new());

	let config = rustls::ClientConfig::builder()
		.with_safe_defaults()
		.with_root_certificates(root_store)
		.with_no_client_auth();

	Arc::new(config)
}


pub struct RustlsBuilder {
	config: Option<Config>,
}

impl TransportBuilder for RustlsBuilder {
	fn build(&self, r: &Role) -> Result<crate::TransportInstance> {
		match r {
			Role::Sealer => Ok(TransportInstance::new(Box::new(Client::from_config(self.config.as_ref())))),
			// Role::Revealer => Ok(TransportInstance::new(Box::new(Client::from_config(self.config.as_ref())))),
			Role::Revealer => Err(Error::Other("not implemented yet".into())),
		}
	}
}


struct Client {
	config: Config,
}

impl Client {
	fn from_config(c: Option<&Config>) -> Self {
		let config = match c {
			Some(config) => config.clone(),
			None => Config::default(),
		};
		return Client{config}
	}
}

#[async_trait]
impl<'a,A> Transport<'a, A> for Client
where
	A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
	async fn wrap(&self, a: A) -> Result<Box<dyn crate::Stream + 'a>> {
		let config = self.config.client_cfg.clone().ok_or(Error::Other("no client config provided".into()))?;
		let connector = TlsConnector::from(config).clone();
		let server_name = "www.rust-lang.org".try_into().unwrap();
		let stream = connector.connect(server_name, a).await?;
		Ok(Box::new(stream))
	}

}


struct Server {
	config:Config,
}

#[async_trait]
impl<'a,A> Transport<'a, A> for Server
where
	A: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a,
{
	async fn wrap(&self, a: A) -> Result<Box<dyn crate::Stream + 'a>> {
		let config = self.config.server_cfg.clone().ok_or(Error::Other("no server config provided".into()))?;
		let acceptor = TlsAcceptor::from(config);
		let mut stream = acceptor.accept(a).await?;
		Ok(Box::new(a))
	}
}


#[cfg(test)]
mod test {
	use super::*;
	use crate::Result;

	#[tokio::test]
	async fn async_tls_rustls_read_write() -> Result<()> {

		let (c, s) = tokio::io::duplex(128);
		// let mut sock = tokio::net::TcpStream::connect("www.rust-lang.org:443").await?;

		tokio::spawn(async move {
			let server = Server {
				config: Config {
					client_cfg: None,
					server_cfg: Some(default_server_config()),
				}
			};
			let wrapped_server_conn = server.wrap(&mut s).await.unwrap();

			let (mut reader, mut writer) = tokio::io::split(wrapped_server_conn);
			let n = tokio::io::copy(&mut reader, &mut writer).await.unwrap();
			writer.flush().await.unwrap();
		});

		let client = Client{
			config: Config {
				client_cfg: Some(default_client_config()),
				server_cfg: None,
			}
		};
		let wrapped_client_conn = client.wrap(&mut c).await?;


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


	/*
	fn load_certs(path: &Path) -> io::Result<Vec<CertificateDer<'static>>> {
		certs(&mut BufReader::new(File::open(path)?)).collect()
	}

	fn load_keys(path: &Path) -> io::Result<PrivateKeyDer<'static>> {
		rsa_private_keys(&mut BufReader::new(File::open(path)?))
			.next()
			.unwrap()
			.map(Into::into)
	}
	*/
}
