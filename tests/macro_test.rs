//! Macro DSL tests.

use https_client::{get, post, Method, Body, HeaderName};

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
fn test_post_macro_simple() -> Result<(), Box<dyn std::error::Error>> {
    let body_data = b"123";
    let req = post!("https://api.binance.com/api/v3/userDataStream", 
        body: Body::from(body_data.as_slice())
    )?;
    
    if req.method() == &Method::Post && req.body().as_ref() == b"123" {
        Ok(())
    } else {
        Err("POST failed".into())
    }
}

#[test]
fn test_post_macro_with_headers() -> Result<(), Box<dyn std::error::Error>> {
    let req = post!("https://api.binance.com/api/v3/userDataStream", 
        headers: {
            "X-MBX-APIKEY" => "my-api-key"
        },
        body: Body::default()
    )?;
    
    if req.headers().len() == 1 && req.headers()[0].name() == &HeaderName::try_from("X-MBX-APIKEY")? {
        Ok(())
    } else {
        Err("Macro failed".into())
    }
}
