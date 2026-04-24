//! Domain model tests for HTTP primitives.

use https_client::{Body, HeaderName, HeaderValue, Method, Status, Header, InitialRequest};

#[test]
fn test_request_creation_success() {
    let method = Method::Get;
    let url = "https://example.com/api";
    
    // Test the Result
    let req = InitialRequest::try_new(method, url).expect("Should parse valid URL");
    
    assert_eq!(req.method(), &Method::Get);
    assert_eq!(req.url().as_str(), "https://example.com/api");
}

#[test]
fn test_request_invalid_scheme() {
    // Should fail because it is not https
    let req = InitialRequest::try_new(Method::Get, "http://example.com");
    assert!(req.is_err());
}

#[test]
fn test_request_builder() {
    let req = InitialRequest::try_new(Method::Post, "https://api.binance.com")
        .unwrap()
        .with_header("Content-Type", "application/json")
        .expect("Should add header")
        .with_body(Body::from(vec![1, 2, 3]));
        
    assert_eq!(req.headers().len(), 1);
    assert_eq!(req.body().len(), 3);
}

#[test]
fn test_method_try_from_success() {
    let m: Method = "GET".try_into().expect("Should parse GET");
    assert_eq!(m, Method::Get);
}

#[test]
fn test_status_try_from_success() {
    let s: Status = "200".try_into().expect("Should parse 200");
    assert_eq!(s, Status::Ok);
}

#[test]
fn test_header_try_from_success() {
    let h: Header = "Content-Type: application/json".try_into().expect("Should parse header");
    assert_eq!(h.name(), &HeaderName::try_from("Content-Type".to_string()).unwrap());
    assert_eq!(h.value(), &HeaderValue::try_from("application/json".to_string()).unwrap());
}

#[test]
fn test_header_name_validation() {
    let valid: Result<HeaderName, _> = "Content-Type".to_string().try_into();
    assert!(valid.is_ok());
    
    let invalid: Result<HeaderName, _> = "Invalid Header".to_string().try_into(); // Space is invalid
    assert!(invalid.is_err());
}

#[test]
fn test_request_monadic_chain_success() {
    let req: Result<InitialRequest, _> = ("POST", "https://example.com").try_into();
    assert!(req.is_ok());
}
