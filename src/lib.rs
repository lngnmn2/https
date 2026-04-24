#![allow(unknown_lints)]
#![deny(tls_models)]

//! # HTTPS Client
//!
//! A mathematically rigorous, type-safe HTTPS client implemented in Rust.
//! This project adheres to Hexagonal Architecture and Type-Driven Design principles.

extern crate alloc;

pub mod domain;
pub mod interpreter;
pub mod macros;

pub use domain::error::HttpError;
pub use domain::request::{Request, SecureRequest, InitialRequest};
pub use domain::response::Response;
pub use domain::header::{Header, HeaderName, HeaderValue};
pub use domain::host::Host;
pub use domain::port::Port;
pub use domain::body::Body;
pub use domain::method::Method;
pub use domain::status::Status;
