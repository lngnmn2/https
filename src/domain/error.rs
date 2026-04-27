//! # Domain Errors
//!
//! Monadic failure model using persistent shared strings.

use thiserror::Error;
use std::rc::Rc;

/// Exhaustive Domain Errors for the HTTPS system.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    /// Failure to parse or validate a URL.
    #[error("URL Error: {0}")]
    UrlError(Rc<str>),

    /// Use of an unsupported or insecure protocol scheme.
    #[error("Insecure Scheme: {0}")]
    InsecureScheme(Rc<str>),

    /// Violation of RFC 7230/9112 header invariants.
    #[error("Protocol Violation (Header): {0}")]
    HeaderError(Rc<str>),

    /// Violation of RFC 7230/9112 method invariants.
    #[error("Protocol Violation (Method): {0}")]
    MethodError(Rc<str>),

    /// Failure to parse or validate a server response.
    #[error("Malformed Response: {0}")]
    ResponseError(Rc<str>),

    /// Failure in the underlying transport or TLS layer.
    #[error("Transport Error: {0}")]
    TransportError(Rc<str>),

    /// Violation of the STG Machine operational semantics.
    #[error("Runtime Error (STG): {0}")]
    RuntimeError(Rc<str>),
}

impl From<url::ParseError> for HttpError {
    fn from(e: url::ParseError) -> Self {
        Self::UrlError(Rc::from(e.to_string()))
    }
}

impl From<std::io::Error> for HttpError {
    fn from(e: std::io::Error) -> Self {
        Self::TransportError(Rc::from(e.to_string()))
    }
}

impl From<openssl::error::ErrorStack> for HttpError {
    fn from(e: openssl::error::ErrorStack) -> Self {
        Self::TransportError(Rc::from(e.to_string()))
    }
}

impl From<openssl::ssl::Error> for HttpError {
    fn from(e: openssl::ssl::Error) -> Self {
        Self::TransportError(Rc::from(format!("{:?}", e)))
    }
}

impl<T: std::fmt::Debug> From<openssl::ssl::HandshakeError<T>> for HttpError {
    fn from(e: openssl::ssl::HandshakeError<T>) -> Self {
        Self::TransportError(Rc::from(format!("{:?}", e)))
    }
}
