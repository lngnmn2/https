//! # Domain Model
//!
//! Pure Algebraic Data Types (ADTs) and Typestate machines representing 
//! the core business logic of the HTTPS system.

/// Exhaustive failure model.
pub mod error;
/// Hostname representation.
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
