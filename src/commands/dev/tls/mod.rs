mod certs;
pub use certs::generate_cert;

use anyhow::Result;
use core::task::{Context, Poll};
use fs::File;
use futures_util::stream::Stream;
use rustls::internal::pemfile;
use rustls::{NoClientAuth, ServerConfig};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::vec::Vec;
use std::{fs, io};
use tokio::net::TcpStream;
use tokio_rustls::{server::TlsStream, TlsAcceptor};

use crate::settings::get_wrangler_home_dir;

// Build TLS configuration
pub(super) fn get_tls_acceptor() -> Result<TlsAcceptor> {
    let home = get_wrangler_home_dir().join("config");
    let cert = home.join("dev-cert.pem");
    let privkey = home.join("dev-privkey.rsa");

    // Load public certificate
    let certs = load_certs(cert)?;

    // Load private key
    let key = load_private_key(privkey)?;

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

fn get_tls_file(file: PathBuf) -> Result<File, io::Error> {
    File::open(&file)
}

// Load public certificate from file.
fn load_certs(file: PathBuf) -> io::Result<Vec<rustls::Certificate>> {
    // Open certificate file.
    let certfile = get_tls_file(file)?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificate.
    pemfile::certs(&mut reader).map_err(|_| io_error("failed to load certificate".into()))
}

// Load private key from file.
fn load_private_key(file: PathBuf) -> io::Result<rustls::PrivateKey> {
    // Open keyfile.
    let keyfile = get_tls_file(file)?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.
    let keys = pemfile::pkcs8_private_keys(&mut reader)
        .map_err(|_| io_error("failed to load private key".into()))?;
    if keys.len() != 1 {
        return Err(io_error("expected a single private key".into()));
    }
    Ok(keys[0].clone())
}
