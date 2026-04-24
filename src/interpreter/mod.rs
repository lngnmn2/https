//! # Interpreter Layer
//!
//! Implementation of the Imperative Shell that executes planned interaction graphs.

/// High-level runner entry point.
pub mod runner;
/// TCP socket adapters.
pub mod socket;
/// TLS channel adapters.
pub mod tls;
/// STG Machine operational core.
pub mod stg;
/// Protocol planning and transducer logic.
pub mod protocol;
