//! Protocol Transducer tests.

use https_client::interpreter::protocol;
use https_client::domain::{SecureRequest, Method, Body, Status, SecurityLevel};
use std::io::Cursor;

#[test]
fn test_plan_pure_logic() -> Result<(), Box<dyn std::error::Error>> {
    let req = SecureRequest::try_new(Method::Get, "https://github.com/")?
        .with_body(Body::default());
    
    let expr = protocol::plan(&req)?;
    
    // Structural property: Interaction begins with Connect
    match expr {
        https_client::interpreter::stg::Expr::Case(..) => Ok(()),
        _ => Err("Expected Case interaction".into()),
    }
}

#[test]
fn test_format_request() -> Result<(), Box<dyn std::error::Error>> {
    let req = SecureRequest::try_new(Method::Post, "https://api.binance.com/api/v3/order")?
        .with_header("X-MBX-APIKEY", "test-key")?
        .with_body(Body::from(b"symbol=BTCUSDT".to_vec()));
        
    let raw = protocol::format_request(&req)?;
    let s = String::from_utf8_lossy(&raw);
    
    if s.contains("POST /api/v3/order HTTP/1.1") &&
       s.contains("Host: api.binance.com") &&
       s.contains("X-MBX-APIKEY: test-key") &&
       s.contains("Content-Length: 14") &&
       s.ends_with("symbol=BTCUSDT") {
        Ok(())
    } else {
        Err(format!("Malformed request: {}", s).into())
    }
}

#[test]
fn test_read_response_simple() -> Result<(), Box<dyn std::error::Error>> {
    let raw = [
        b"HTTP/1.1 200 OK\r\n".as_slice(),
        b"Content-Type: text/plain\r\n".as_slice(),
        b"Content-Length: 5\r\n".as_slice(),
        b"Strict-Transport-Security: max-age=31536000\r\n".as_slice(),
        b"X-Content-Type-Options: nosniff\r\n".as_slice(),
        b"Content-Security-Policy: default-src 'self'\r\n".as_slice(),
        b"\r\n".as_slice(),
        b"Hello".as_slice(),
    ].concat();
    
    let reader = Cursor::new(raw);
    let (_, resp) = protocol::read_response_pure(reader, SecurityLevel::Strict)?;

    if resp.status() == Status::Ok && resp.body().as_ref() == b"Hello" {
        Ok(())
    } else {
        Err("Response mismatch".into())
    }
}
