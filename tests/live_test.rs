//! Live tests against external APIs.

use https_client::{InitialRequest, Body};
use https_client::Method;
use https_client::interpreter::runner;
use https_client::Status;

#[test]
fn test_live_github_https_get() {
    let req = InitialRequest::try_new(Method::Get, "https://github.com/")
        .unwrap()
        .with_body(Body::default());
    
    let resp = runner::execute(&req).expect("HTTPS GET failed");
    assert_eq!(resp.status(), Status::Ok);
}

#[test]
fn test_live_invalid_certificate_rejected() {
    // expired.badssl.com is a well-known endpoint for testing invalid certs
    let req = InitialRequest::try_new(Method::Get, "https://expired.badssl.com/")
        .unwrap()
        .with_body(Body::default());
    
    let res = runner::execute(&req);
    
    // It must fail due to TLS handshake/verification error
    assert!(res.is_err());
}
