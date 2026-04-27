//! # Interpreter Layer (Imperative Shell)
//!
//! Implementation of the STG runtime and protocol transducer.

/// High-level interaction runner.
pub mod runner;
/// TCP socket adapters.
pub mod socket;
/// TLS adapter.
pub mod tls;
/// GHC STG Machine implementation.
pub mod stg;
/// Interaction transducer (DSL -> Initial Algebra).
pub mod protocol;
