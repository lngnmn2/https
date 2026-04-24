//! # TCP Socket Adapter
//!
//! Imperative Shell: Connects to the physical network.

use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use crate::domain::error::HttpError;

/// Establishes a TCP connection with a strict timeout.
pub fn connect_tcp(host: &str, port: u16) -> Result<TcpStream, HttpError> {
    let addr_str = format!("{}:{}", host, port);
    let timeout = Duration::from_secs(10);
    
    let addr = addr_str.to_socket_addrs()?
        .next()
        .ok_or_else(|| HttpError::TransportError(format!("DNS Resolution Failed: {}", host)))?;

    let stream = TcpStream::connect_timeout(&addr, timeout)?;
    Ok(stream)
}
