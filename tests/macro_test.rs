//! Macro DSL behavioral tests.

use https_client::{get, post, Method, Body, HeaderName, SecurityLevel};

#[test]
fn test_get_macro_simple() -> Result<(), Box<dyn std::error::Error>> {
    let req = get!("https://api.binance.com/api/v3/ping")?;
    if req.method() == &Method::Get && req.url().as_str() == "https://api.binance.com/api/v3/ping" {
        Ok(())
    } else {
        Err("Macro failed".into())
    }
}

#[test]
fn test_get_macro_with_headers() -> Result<(), Box<dyn std::error::Error>> {
    let req = get!("https://api.binance.com/api/v3/ping", 
        headers: {
            "X-MBX-APIKEY" => "my-api-key"
        }
    )?;
    
    if req.headers().len() == 1 && req.headers()[0].name() == &HeaderName::try_from("X-MBX-APIKEY")? {
        Ok(())
    } else {
        Err("Header mismatch".into())
    }
}

#[test]
fn test_get_macro_security_level() -> Result<(), Box<dyn std::error::Error>> {
    let req = get!("https://api.hyperliquid.xyz/info", security: SecurityLevel::Standard)?;
    if req.security_level() == SecurityLevel::Standard {
        Ok(())
    } else {
        Err("Security level mismatch".into())
    }
}

#[test]
fn test_post_macro_with_headers_and_body() -> Result<(), Box<dyn std::error::Error>> {
    let req = post!("https://api.binance.com/api/v3/order", 
        headers: {
            "X-MBX-APIKEY" => "my-api-key"
        },
        body: Body::from(b"test".as_slice())
    )?;
    
    if req.method() == &Method::Post && req.body().as_ref() == b"test" {
        Ok(())
    } else {
        Err("Macro failed".into())
    }
}
