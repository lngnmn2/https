//! # Domain Model (Pure Core)
//!
//! Pure Algebraic Data Types (ADTs) and Typestate machines.

/// Exhaustive failure model.
pub mod error;
/// Hostname representation (RFC 1123).
pub mod host;
/// TCP port representation.
pub mod port;
/// HTTP Body representation.
pub mod body;
/// HTTP Method representation.
pub mod method;
/// HTTP Status representation.
pub mod status;
/// Atomic HTTP headers.
pub mod header;
/// Typestate-enforced Request model.
pub mod request;
/// Domain Response model.
pub mod response;

pub use error::HttpError;
pub use host::Host;
pub use port::Port;
pub use body::Body;
pub use method::Method;
pub use status::Status;
pub use header::{Header, HeaderName, HeaderValue, SecurityLevel};
pub use request::{SecureRequest, Request, InitialRequest};
pub use response::Response;
