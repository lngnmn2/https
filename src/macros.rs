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

/// Construct a POST request.
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
