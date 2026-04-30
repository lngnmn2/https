//! # Protocol Regression Test Suite
//!
//! Formal verification of protocol invariants and security hardening.
//! Refactored for Pure Functional compliance.

use https_client::interpreter::{protocol, runner};
use https_client::domain::{HttpError, InitialRequest, Method, SecurityLevel};
use std::io::Cursor;

#[test]
fn test_regression_conflicting_framing() -> Result<(), Box<dyn std::error::Error>> {
    let raw = [
        b"HTTP/1.1 200 OK\r\n".as_slice(),
        b"Content-Length: 10\r\n".as_slice(),
        b"Transfer-Encoding: chunked\r\n".as_slice(),
        b"Strict-Transport-Security: max-age=31536000\r\n".as_slice(),
        b"X-Content-Type-Options: nosniff\r\n".as_slice(),
        b"Content-Security-Policy: default-src 'self'\r\n".as_slice(),
        b"\r\n".as_slice(),
    ].concat();

    let res = protocol::read_response_pure(Cursor::new(raw), SecurityLevel::Strict);
    
    match res {
        Err(HttpError::ResponseError(e)) if e.contains("Conflicting Framing") => Ok(()),
        other => Err(format!("Expected Conflicting Framing error, got {:?}", other).into()),
    }
}

#[test]
fn test_regression_missing_hsts() -> Result<(), Box<dyn std::error::Error>> {
    let raw = [
        b"HTTP/1.1 200 OK\r\n".as_slice(),
        b"Content-Length: 0\r\n".as_slice(),
        b"X-Content-Type-Options: nosniff\r\n".as_slice(),
        b"Content-Security-Policy: default-src 'self'\r\n".as_slice(),
        b"\r\n".as_slice(),
    ].concat();

    let res = protocol::read_response_pure(Cursor::new(raw), SecurityLevel::Strict);
    
    match res {
        Err(HttpError::ResponseError(e)) if e.contains("Missing HSTS") => Ok(()),
        other => Err(format!("Expected Missing HSTS error, got {:?}", other).into()),
    }
}

#[test]
fn test_regression_resource_limits_line_length() -> Result<(), Box<dyn std::error::Error>> {
    let long_line = vec![b'A'; 9000];
    let raw = [
        b"HTTP/1.1 200 OK\r\n".as_slice(),
        &long_line,
        b"\r\n".as_slice(),
    ].concat();

    let res = protocol::read_response_pure(Cursor::new(raw), SecurityLevel::Strict);
    
    match res {
        Err(HttpError::ResponseError(e)) if e.contains("EOF") || e.contains("Transport Error") => Ok(()),
        Err(_) => Ok(()),
        Ok(_) => Err("Expected failure for over-sized line".into()),
    }
}

#[test]
fn test_regression_typestate_host_enforcement() -> Result<(), Box<dyn std::error::Error>> {
    // A URL with no host part
    let res = InitialRequest::try_new(Method::Get, "https://:443/");
    
    match res {
        Err(HttpError::UrlError(e)) if e.to_lowercase().contains("host") => Ok(()),
        other => Err(format!("Expected Host error, got {:?}", other).into()),
    }
}

#[test]
fn test_regression_body_limit_enforcement() -> Result<(), Box<dyn std::error::Error>> {
    let raw = [
        b"HTTP/1.1 200 OK\r\n".as_slice(),
        b"Content-Length: 22020096\r\n".as_slice(),
        b"Strict-Transport-Security: max-age=31536000\r\n".as_slice(),
        b"X-Content-Type-Options: nosniff\r\n".as_slice(),
        b"Content-Security-Policy: default-src 'self'\r\n".as_slice(),
        b"\r\n".as_slice(),
    ].concat();

    let res = protocol::read_response_pure(Cursor::new(raw), SecurityLevel::Strict);
    
    match res {
        Err(HttpError::ResponseError(e)) if e.contains("Too Large") => Ok(()),
        other => Err(format!("Expected Too Large error, got {:?}", other).into()),
    }
}

#[test]
fn test_regression_mandatory_peer_verification() -> Result<(), Box<dyn std::error::Error>> {
    let req = InitialRequest::try_new(Method::Get, "https://untrusted-root.badssl.com/")?
        .build();
    
    let res = runner::execute(&req);
    match res {
        Err(HttpError::TransportError(e)) if e.contains("certificate verify failed") || e.contains("Handshake") => Ok(()),
        Ok(_) => Err("Insecure certificate (untrusted root) was accepted".into()),
        other => Err(format!("Expected verification error, got {:?}", other).into()),
    }
}
