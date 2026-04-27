//! # HTTP Method Domain Model
//!
//! Ref: https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods

use super::error::HttpError;
use std::rc::Rc;

/// Validated HTTP request methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Method {
    /// GET method requests a representation of the specified resource.
    #[default] Get, 
    /// POST method is used to submit an entity to the specified resource.
    Post, 
    /// PUT method replaces all current representations of the target resource with the request payload.
    Put, 
    /// DELETE method deletes the specified resource.
    Delete, 
    /// HEAD method asks for a response identical to that of a GET request, but without the response body.
    Head, 
    /// OPTIONS method is used to describe the communication options for the target resource.
    Options, 
    /// CONNECT method establishes a tunnel to the server identified by the target resource.
    Connect, 
    /// TRACE method performs a message loop-back test along the path to the target resource.
    Trace, 
    /// PATCH method is used to apply partial modifications to a resource.
    Patch,
}

impl Method {
    /// Converts a Method to its standard string representation.
    pub const fn as_str(self) -> &'static str {
        match self {
            Method::Get => "GET", Method::Post => "POST", Method::Put => "PUT",
            Method::Delete => "DELETE", Method::Head => "HEAD", Method::Options => "OPTIONS",
            Method::Connect => "CONNECT", Method::Trace => "TRACE", Method::Patch => "PATCH",
        }
    }
}

impl TryFrom<&str> for Method {
    type Error = HttpError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::Get), "POST" => Ok(Method::Post), "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete), "HEAD" => Ok(Method::Head), "OPTIONS" => Ok(Method::Options),
            "CONNECT" => Ok(Method::Connect), "TRACE" => Ok(Method::Trace), "PATCH" => Ok(Method::Patch),
            _ => Err(HttpError::MethodError(Rc::from(s))),
        }
    }
}

impl From<Method> for &'static str { fn from(m: Method) -> Self { m.as_str() } }
impl std::fmt::Display for Method { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.as_str()) } }
