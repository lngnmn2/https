//! # TLS Adapter
//!
//! Imperative Shell: Establishes a secure financial-grade channel.

use openssl::ssl::{SslConnector, SslMethod, SslStream, SslVerifyMode, SslOptions, SslVersion};
use std::net::TcpStream;
use crate::domain::error::HttpError;

/// Strong TLS Configuration for Financial Applications.
#[derive(Debug, Clone, Copy, Default)]
pub struct Config;

impl Config {
    /// Returns a secure SslConnector with modern defaults.
    /// 
    /// # Errors
    /// Returns `HttpError::TransportError` if the configuration fails.
    pub fn secure_connector() -> Result<SslConnector, HttpError> {
        let mut b = SslConnector::builder(SslMethod::tls())
            .map_err(|e| HttpError::TransportError(format!("TLS Init: {}", e)))?;
        
        b.set_min_proto_version(Some(SslVersion::TLS1_2))
            .map_err(|e| HttpError::TransportError(format!("TLS Version: {}", e)))?;

        let ciphers = "ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:\
                       ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:\
                       ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305";
        b.set_cipher_list(ciphers)
            .map_err(|e| HttpError::TransportError(format!("TLS Cipher: {}", e)))?;

        b.set_verify(SslVerifyMode::PEER);
        b.set_options(SslOptions::NO_COMPRESSION);
        b.set_default_verify_paths()
            .map_err(|e| HttpError::TransportError(format!("TLS Cert Path: {}", e)))?;

        Ok(b.build())
    }
}

/// Establishes a secure TLS connection over a TCP stream.
/// 
/// # Errors
/// Returns `HttpError::TransportError` if the handshake fails.
pub fn connect_tls(host: &str, stream: TcpStream) -> Result<SslStream<TcpStream>, HttpError> {
    let connector = Config::secure_connector()?;
    connector.connect(host, stream)
        .map_err(|e| HttpError::TransportError(format!("TLS Handshake: {}", e)))
}
