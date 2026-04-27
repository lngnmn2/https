//! # Live API Integration Tests
//! 
//! Verification against production financial endpoints.

use https_client::{InitialRequest, Method, Body, Status, SecurityLevel, post_json};
use https_client::interpreter::runner;
use serde::Serialize;

#[test]
fn test_live_binance_ping() -> Result<(), Box<dyn std::error::Error>> {
    // Ref: https://binance-docs.github.io/apidocs/futures/en/#test-connectivity
    let req = InitialRequest::try_new(Method::Get, "https://fapi.binance.com/fapi/v1/ping")?
        .with_body(Body::default());
        
    let resp = runner::execute(&req)?;
    
    if resp.status() == Status::Ok {
        Ok(())
    } else {
        Err(format!("Binance Ping failed: {}", resp.status()).into())
    }
}

#[test]
fn test_live_binance_exchange_info() -> Result<(), Box<dyn std::error::Error>> {
    // Ref: curl "https://fapi.binance.com/fapi/v1/exchangeInfo"
    let req = InitialRequest::try_new(Method::Get, "https://fapi.binance.com/fapi/v1/exchangeInfo")?
        .with_body(Body::default());
        
    let resp = runner::execute(&req)?;
    
    if resp.status() == Status::Ok {
        Ok(())
    } else {
        Err(format!("Binance Exchange Info failed: {}", resp.status()).into())
    }
}

#[test]
fn test_live_binance_user_data_stream_json() -> Result<(), Box<dyn std::error::Error>> {
    // Ref: curl -X POST "https://fapi.binance.com/fapi/v1/listenKey"
    #[derive(Serialize)]
    struct Dummy { foo: &'static str }
    
    let req = post_json!("https://fapi.binance.com/fapi/v1/listenKey", json: &Dummy { foo: "bar" })?
        .with_security_level(SecurityLevel::Minimal); // Binance listenKey lacks HSTS and Nosniff
    
    let resp = runner::execute(&req)?;
    
    // 401 is expected as we don't provide an API key, proving protocol success.
    if resp.status() == Status::Unauthorized || resp.status().code() == 401 || resp.status().code() == 400 {
        Ok(())
    } else {
        Err(format!("Binance ListenKey failed with unexpected status: {}", resp.status()).into())
    }
}

#[test]
fn test_live_hyperliquid_info() -> Result<(), Box<dyn std::error::Error>> {
    // Ref: https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint
    let body_data = b"{\"type\": \"meta\"}";
    let req = InitialRequest::try_new(Method::Post, "https://api.hyperliquid.xyz/info")?
        .with_header("Content-Type", "application/json")?
        .with_security_level(SecurityLevel::Minimal) // Hyperliquid lacks HSTS and Nosniff
        .with_body(Body::from(body_data.as_slice()));
        
    let resp = runner::execute(&req)?;
    
    // Financial grade check: ensure it's not a 404 or transport error
    if resp.status() == Status::Ok || resp.status().code() == 400 {
        Ok(())
    } else {
        Err(format!("Hyperliquid Meta failed with unexpected status: {}", resp.status()).into())
    }
}
