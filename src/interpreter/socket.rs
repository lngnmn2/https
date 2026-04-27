//! # TCP Socket Adapter
//!
//! Side-effect heavy adapter for networking.

use crate::domain::HttpError;
use std::net::TcpStream;

/// Establishes a TCP connection.
pub fn connect_tcp(host: &str, port: u16) -> Result<TcpStream, HttpError> {
    TcpStream::connect((host, port)).map_err(Into::into)
}
