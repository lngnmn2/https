//! # TLS Adapter
//!
//! Side-effect heavy adapter for secure channels.

use crate::domain::HttpError;
use openssl::ssl::{SslMethod, SslConnector, SslStream, SslVerifyMode, SslConnectorBuilder};
use std::net::TcpStream;
use std::io::Write;
use std::rc::Rc;

/// Performs a TLS handshake.
pub fn connect_tls(host: &str, stream: TcpStream) -> Result<SslStream<TcpStream>, HttpError> {
    let builder = configure_connector(SslConnector::builder(SslMethod::tls())?);
    builder.build().connect(host, stream).map_err(Into::into)
}

fn configure_connector(builder: SslConnectorBuilder) -> SslConnectorBuilder {
    // Note: External builder API necessitates local mutation for configuration.
    // We isolate this at the infrastructure boundary.
    let mut b = builder;
    b.set_verify(SslVerifyMode::PEER);
    let _ = b.set_min_proto_version(Some(openssl::ssl::SslVersion::TLS1_2));
    let _ = b.set_cipher_list("ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256");
    b
}

/// Pure functional wrapper for TLS write operations.
pub fn write_all_pure(stream: SslStream<TcpStream>, data: &[u8]) -> Result<SslStream<TcpStream>, HttpError> {
    let mut s = stream;
    s.write_all(data).and_then(|_| s.flush()).map(|_| s).map_err(|e| HttpError::TransportError(Rc::from(e.to_string())))
}
