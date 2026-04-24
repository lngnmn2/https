//! Macro DSL tests.

use https_client::{get, post, Method, Body, HeaderName};

#[test]
fn test_get_macro_simple() {
    let req = get!("https://api.binance.com/api/v3/ping").expect("Macro failed");
    assert_eq!(req.method(), &Method::Get);
    assert_eq!(req.url().as_str(), "https://api.binance.com/api/v3/ping");
    assert!(req.body().is_empty());
}

#[test]
fn test_get_macro_with_headers() {
    let req = get!("https://api.binance.com/api/v3/ping", 
        headers: {
            "X-MBX-APIKEY" => "my-api-key",
            "Content-Type" => "application/json"
        }
    ).expect("Macro failed");
    
    assert_eq!(req.headers().len(), 2);
    assert_eq!(req.headers()[0].name(), &HeaderName::try_from("X-MBX-APIKEY".to_string()).unwrap());
}

#[test]
fn test_post_macro_simple() {
    let body_data = vec![1, 2, 3];
    let req = post!("https://api.binance.com/api/v3/userDataStream", 
        body: Body::from(body_data)
    ).expect("Macro failed");
    
    assert_eq!(req.method(), &Method::Post);
    assert_eq!(req.body().as_ref(), &[1, 2, 3]);
}

#[test]
fn test_post_macro_with_headers() {
    let req = post!("https://api.binance.com/api/v3/userDataStream", 
        headers: {
            "X-MBX-APIKEY" => "my-api-key"
        },
        body: Body::default()
    ).expect("Macro failed");
    
    assert_eq!(req.headers().len(), 1);
    assert_eq!(req.headers()[0].name(), &HeaderName::try_from("X-MBX-APIKEY".to_string()).unwrap());
}
