use core::task::{Context, Poll};
use fs::File;
use futures_util::stream::Stream;
use rustls::internal::pemfile;
use rustls::{NoClientAuth, ServerConfig};
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::vec::Vec;
use std::{fs, io};
use tokio::net::TcpStream;
use tokio_rustls::{server::TlsStream, TlsAcceptor};

// Build TLS configuration
pub(super) fn get_tls_acceptor() -> Result<TlsAcceptor, io::Error> {
    // Load public certificate
    let certs = load_certs("sample.pem")?;

    // Load private key
    let key = load_private_key("sample.rsa")?;

    // Do not use client certificate authentication.
    let mut cfg = ServerConfig::new(NoClientAuth::new());

    // Select a certificate to use.
    cfg.set_single_cert(certs, key)
        .map_err(|e| io_error(format!("{}", e)))?;

    Ok(TlsAcceptor::from(Arc::new(cfg)))
}

pub(super) fn io_error(err: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}

pub(super) struct HyperAcceptor<'a> {
    pub(super) acceptor:
        Pin<Box<dyn Stream<Item = Result<TlsStream<TcpStream>, io::Error>> + Send + 'a>>,
}

impl hyper::server::accept::Accept for HyperAcceptor<'_> {
    type Conn = TlsStream<TcpStream>;
    type Error = io::Error;

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        Pin::new(&mut self.acceptor).poll_next(cx)
    }
}

fn get_tls_file(filename: &str) -> Result<File, io::Error> {
    let path = Path::new(&dirs::home_dir().unwrap())
        .join("Documents/work/wrangler/tls")
        .join(&filename);
    File::open(&path).map_err(|e| io_error(format!("failed to open {}: {}", filename, e)))
}

// Load public certificate from file.
fn load_certs(filename: &str) -> io::Result<Vec<rustls::Certificate>> {
    // Open certificate file.
    let certfile = get_tls_file(&filename)?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificate.
    pemfile::certs(&mut reader).map_err(|_| io_error("failed to load certificate".into()))
}

// Load private key from file.
fn load_private_key(filename: &str) -> io::Result<rustls::PrivateKey> {
    // Open keyfile.
    let keyfile = get_tls_file(&filename)?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.
    let keys = pemfile::rsa_private_keys(&mut reader)
        .map_err(|_| io_error("failed to load private key".into()))?;
    if keys.len() != 1 {
        return Err(io_error("expected a single private key".into()));
    }
    Ok(keys[0].clone())
}
