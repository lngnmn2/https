//! Live connectivity tests (Using GitHub as a stable target).

use https_client::{InitialRequest, Method, Body, Status};
use https_client::interpreter::runner;

#[test]
fn test_live_github_https_get() -> Result<(), Box<dyn std::error::Error>> {
    let req = InitialRequest::try_new(Method::Get, "https://github.com/")?
        .with_body(Body::default());
        
    let resp = runner::execute(&req)?;
    
    if resp.status() == Status::Ok {
        Ok(())
    } else {
        Err(format!("Expected 200 OK, got {}", resp.status()).into())
    }
}

#[test]
fn test_live_invalid_certificate_rejected() -> Result<(), Box<dyn std::error::Error>> {
    // expired.badssl.com should be rejected by our mandatory peer verification.
    let req = InitialRequest::try_new(Method::Get, "https://expired.badssl.com/")?
        .with_body(Body::default());
        
    let res = runner::execute(&req);
    
    if res.is_err() {
        Ok(())
    } else {
        Err("Invalid certificate should have been rejected".into())
    }
}
