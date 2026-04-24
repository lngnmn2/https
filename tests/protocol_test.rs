//! Protocol parser and formatter tests.

use https_client::{Host, Port, SecureRequest, Status};
use https_client::Method;
use https_client::interpreter::protocol;
use https_client::interpreter::stg::Expr;
use std::io::Cursor;

#[test]
fn test_plan_pure_logic() {
    let req = SecureRequest::try_new(Method::Get, "https://api.binance.com/api/v3/ping")
        .unwrap();
        
    let plan = protocol::plan(&req).expect("Planning failed");
    
    // Validate the STG Expr structure: Case(OpConnect, ...)
    if let Expr::Case(target, _) = plan {
        if let Expr::OpConnect(host, port) = *target {
            assert_eq!(host, Host::try_from("api.binance.com").unwrap());
            assert_eq!(port, Port::from(443));
        } else {
            panic!("Expected OpConnect as Case target");
        }
    } else {
        panic!("Expected Case as top-level Expr");
    }
}

#[test]
fn test_format_request() {
    let req = SecureRequest::try_new(Method::Get, "https://example.com/api").unwrap()
        .with_header("User-Agent", "TestClient/1.0").expect("Should add header");

    let bytes = protocol::format_request(&req).expect("Formatting failed");
    let s = String::from_utf8(bytes).unwrap();

    // Check method and path
    assert!(s.starts_with("GET /api HTTP/1.1\r\n"));
    
    // Check mandatory headers
    assert!(s.contains("Host: example.com\r\n"));
    assert!(s.contains("Connection: close\r\n"));
    
    // Check custom headers
    assert!(s.contains("User-Agent: TestClient/1.0\r\n"));
    
    // Check end of headers
    assert!(s.ends_with("\r\n\r\n"));
}

#[test]
fn test_read_response_simple() {
    let mut raw = Vec::new();
    raw.extend_from_slice(b"HTTP/1.1 200 OK\r\n");
    raw.extend_from_slice(b"Strict-Transport-Security: max-age=31536000\r\n");
    raw.extend_from_slice(b"X-Content-Type-Options: nosniff\r\n");
    raw.extend_from_slice(b"Content-Security-Policy: default-src 'self'\r\n");
    raw.extend_from_slice(b"Content-Type: text/plain\r\n");
    raw.extend_from_slice(b"Content-Length: 5\r\n");
    raw.extend_from_slice(b"\r\n");
    raw.extend_from_slice(b"Hello");
    
    let mut reader = Cursor::new(raw);
    
    let resp = protocol::read_response(&mut reader).expect("Should parse valid response");

    assert_eq!(resp.status(), Status::Ok);
    assert_eq!(resp.body().as_ref(), b"Hello");
}
