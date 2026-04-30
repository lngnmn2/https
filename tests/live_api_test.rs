//! # Live API Integration Tests
//! 
//! Verification against production financial endpoints using the high-level DSL.

use https_client::{get, post, Body, Status, SecurityLevel};
use https_client::interpreter::runner;

#[test]
fn test_live_binance_ping() -> Result<(), Box<dyn std::error::Error>> {
    let req = get!("https://fapi.binance.com/fapi/v1/ping")?;
    let resp = runner::execute(&req)?;
    
    if resp.status() == Status::Ok {
        Ok(())
    } else {
        Err(format!("Binance Ping failed: {}", resp.status()).into())
    }
}

#[test]
fn test_live_hyperliquid_info() -> Result<(), Box<dyn std::error::Error>> {
    let body_data = b"{\"type\": \"meta\"}";
    let req = post!("https://api.hyperliquid.xyz/info", 
        headers: {
            "Content-Type" => "application/json"
        },
        body: Body::from(body_data.as_slice())
    )?;
    
    // Hyperliquid lacks some standard security headers
    let req_final = req.with_security_level(SecurityLevel::None);
        
    let resp = runner::execute(&req_final)?;
    
    if resp.status() == Status::Ok || resp.status().code() == 400 {
        Ok(())
    } else {
        Err(format!("Hyperliquid Meta failed with unexpected status: {}", resp.status()).into())
    }
}
