//! Domain logic tests.

use https_client::{InitialRequest, Method, Header, Status, HeaderName, HeaderValue};

#[test]
fn test_request_creation_success() -> Result<(), Box<dyn std::error::Error>> {
    let method = Method::Get;
    let url = "https://api.binance.com/api/v3/ping";
    let req = InitialRequest::try_new(method, url)?;
    
    if req.method() == &Method::Get && req.url().as_str() == url {
        Ok(())
    } else {
        Err("Request mismatch".into())
    }
}

#[test]
fn test_request_invalid_scheme() -> Result<(), Box<dyn std::error::Error>> {
    let res = InitialRequest::try_new(Method::Get, "http://api.binance.com/");
    if res.is_err() {
        Ok(())
    } else {
        Err("Should fail for http".into())
    }
}

#[test]
fn test_request_builder() -> Result<(), Box<dyn std::error::Error>> {
    let req = InitialRequest::try_new(Method::Get, "https://api.binance.com/")?
        .with_header("X-MBX-APIKEY", "test")?;
        
    if req.headers().len() == 1 {
        Ok(())
    } else {
        Err("Header not added".into())
    }
}

#[test]
fn test_method_try_from_success() -> Result<(), Box<dyn std::error::Error>> {
    let m: Method = "GET".try_into()?;
    if m == Method::Get {
        Ok(())
    } else {
        Err("Method mismatch".into())
    }
}

#[test]
fn test_status_try_from_success() -> Result<(), Box<dyn std::error::Error>> {
    let s: Status = "200".try_into()?;
    if s == Status::Ok {
        Ok(())
    } else {
        Err("Status mismatch".into())
    }
}

#[test]
fn test_header_try_from_success() -> Result<(), Box<dyn std::error::Error>> {
    let h: Header = "Content-Type: application/json".try_into()?;
    if h.name() == &HeaderName::try_from("Content-Type")? &&
       h.value() == &HeaderValue::try_from("application/json")? {
        Ok(())
    } else {
        Err("Header mismatch".into())
    }
}

#[test]
fn test_header_name_validation() -> Result<(), Box<dyn std::error::Error>> {
    let valid: Result<HeaderName, _> = "Content-Type".try_into();
    let invalid: Result<HeaderName, _> = "Invalid Header".try_into(); // Space is invalid
    
    if valid.is_ok() && invalid.is_err() {
        Ok(())
    } else {
        Err("Validation failed".into())
    }
}

#[test]
fn test_request_monadic_chain_success() -> Result<(), Box<dyn std::error::Error>> {
    let res = InitialRequest::try_new(Method::Get, "https://api.binance.com/")
        .and_then(|r| r.with_header("X-Test", "Value"));
        
    if res.is_ok() {
        Ok(())
    } else {
        Err("Chain failed".into())
    }
}
