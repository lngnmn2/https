use thiserror::Error;

/// Exhaustive Domain Errors for the HTTPS system.
/// Parameterized by relevant values to ensure transparency in monadic propagation.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    /// Failure to parse or validate a URL.
    #[error("URL Error: {0}")]
    UrlError(String),

    /// Use of an unsupported or insecure protocol scheme.
    #[error("Insecure Scheme: {0}")]
    InsecureScheme(String),

    /// Violation of RFC 7230/9112 header invariants.
    #[error("Protocol Violation (Header): {0}")]
    HeaderError(String),

    /// Violation of RFC 7230/9112 method invariants.
    #[error("Protocol Violation (Method): {0}")]
    MethodError(String),

    /// Failure to parse or validate a server response.
    #[error("Malformed Response: {0}")]
    ResponseError(String),

    /// Failure in the underlying transport or TLS layer.
    #[error("Transport Error: {0}")]
    TransportError(String),

    /// Violation of the STG Machine operational semantics.
    #[error("Runtime Error (STG): {0}")]
    RuntimeError(String),
}

impl From<url::ParseError> for HttpError {
    fn from(e: url::ParseError) -> Self {
        Self::UrlError(e.to_string())
    }
}

impl From<std::io::Error> for HttpError {
    fn from(e: std::io::Error) -> Self {
        Self::TransportError(e.to_string())
    }
}

impl From<openssl::error::ErrorStack> for HttpError {
    fn from(e: openssl::error::ErrorStack) -> Self {
        Self::TransportError(e.to_string())
    }
}
