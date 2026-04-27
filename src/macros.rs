//! # DSL Macros
//! 
//! Declarative macros for constructing HTTPS requests.

/// Construct a GET request.
/// 
/// # Example
/// ```
/// use https_client::get;
/// let req = get!("https://api.binance.com/api/v3/ping");
/// ```
#[macro_export]
macro_rules! get {
    ($url:expr) => {
        $crate::SecureRequest::try_new($crate::Method::Get, $url)
            .map(|r| r.with_body($crate::Body::default()))
    };
    ($url:expr, headers: { $($name:expr => $val:expr),* $(,)? }) => {
        $crate::SecureRequest::try_new($crate::Method::Get, $url)
            $(.and_then(|r| r.with_header($name, $val)))*
            .map(|r| r.with_body($crate::Body::default()))
    };
}

/// Construct a POST request with a raw body.
/// 
/// # Example
/// ```
/// use https_client::post;
/// let req = post!("https://api.binance.com/api/v3/userDataStream", body: vec![]);
/// ```
#[macro_export]
macro_rules! post {
    ($url:expr, body: $body:expr) => {
        $crate::SecureRequest::try_new($crate::Method::Post, $url)
            .map(|r| r.with_body($body))
    };
    ($url:expr, headers: { $($name:expr => $val:expr),* $(,)? }, body: $body:expr) => {
        $crate::SecureRequest::try_new($crate::Method::Post, $url)
            $(.and_then(|r| r.with_header($name, $val)))*
            .map(|r| r.with_body($body))
    };
}

/// Construct a POST request with a JSON body.
/// 
/// # Example
/// ```
/// use https_client::post_json;
/// use serde::Serialize;
/// #[derive(Serialize)]
/// struct Payload { symbol: &'static str }
/// let req = post_json!("https://fapi.binance.com/fapi/v1/listenKey", json: &Payload { symbol: "BTCUSDT" });
/// ```
#[macro_export]
macro_rules! post_json {
    ($url:expr, json: $data:expr) => {
        $crate::SecureRequest::try_new($crate::Method::Post, $url)
            .and_then(|r| r.with_json($data))
    };
    ($url:expr, headers: { $($name:expr => $val:expr),* $(,)? }, json: $data:expr) => {
        $crate::SecureRequest::try_new($crate::Method::Post, $url)
            $(.and_then(|r| r.with_header($name, $val)))*
            .and_then(|r| r.with_json($data))
    };
}
