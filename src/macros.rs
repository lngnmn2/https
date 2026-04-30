//! # DSL Macros
//! 
//! Declarative macros for high-level HTTPS interaction.

/// Construct a GET request with optional headers and security level.
/// 
/// # Examples
/// ```
/// use https_client::{get, SecurityLevel};
/// let req1 = get!("https://api.binance.com/api/v3/ping");
/// let req2 = get!("https://api.binance.com/api/v3/ping", 
///     headers: { "X-MBX-APIKEY" => "test" });
/// let req3 = get!("https://api.hyperliquid.xyz/info", 
///     security: SecurityLevel::None);
/// ```
#[macro_export]
macro_rules! get {
    ($url:expr) => {
        $crate::SecureRequest::try_new($crate::Method::Get, $url)
            .map(|r| r.build())
    };
    ($url:expr, headers: { $($name:expr => $val:expr),* $(,)? }) => {
        $crate::SecureRequest::try_new($crate::Method::Get, $url)
            $(.and_then(|r| r.with_header($name, $val)))*
            .map(|r| r.build())
    };
    ($url:expr, security: $level:expr) => {
        $crate::SecureRequest::try_new($crate::Method::Get, $url)
            .map(|r| r.with_security_level($level).build())
    };
    ($url:expr, headers: { $($name:expr => $val:expr),* $(,)? }, security: $level:expr) => {
        $crate::SecureRequest::try_new($crate::Method::Get, $url)
            $(.and_then(|r| r.with_header($name, $val)))*
            .map(|r| r.with_security_level($level).build())
    };
    ($url:expr, security: $level:expr, headers: { $($name:expr => $val:expr),* $(,)? }) => {
        $crate::SecureRequest::try_new($crate::Method::Get, $url)
            .map(|r| r.with_security_level($level))
            $(.and_then(|r| r.with_header($name, $val)))*
            .map(|r| r.build())
    };
}

/// Construct a POST request with a body and optional headers/security.
/// 
/// # Examples
/// ```
/// use https_client::{post, Body, SecurityLevel};
/// let req1 = post!("https://api.hyperliquid.xyz/info", body: Body::from(b"test".to_vec()));
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
    ($url:expr, security: $level:expr, body: $body:expr) => {
        $crate::SecureRequest::try_new($crate::Method::Post, $url)
            .map(|r| r.with_security_level($level).with_body($body))
    };
    ($url:expr, headers: { $($name:expr => $val:expr),* $(,)? }, security: $level:expr, body: $body:expr) => {
        $crate::SecureRequest::try_new($crate::Method::Post, $url)
            $(.and_then(|r| r.with_header($name, $val)))*
            .map(|r| r.with_security_level($level).with_body($body))
    };
    ($url:expr, security: $level:expr, headers: { $($name:expr => $val:expr),* $(,)? }, body: $body:expr) => {
        $crate::SecureRequest::try_new($crate::Method::Post, $url)
            .map(|r| r.with_security_level($level))
            $(.and_then(|r| r.with_header($name, $val)))*
            .map(|r| r.with_body($body))
    };
}
