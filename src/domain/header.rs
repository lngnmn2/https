//! # HTTP Header Domain Model
//!
//! Ref: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers
//! Enforces RFC 7230 invariants via "Correctness by Construction".

use super::error::HttpError;
use std::rc::Rc;

/// Validated Header Name (RFC 7230 token).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderName(Rc<str>);

impl TryFrom<Rc<str>> for HeaderName {
    type Error = HttpError;
    fn try_from(s: Rc<str>) -> Result<Self, Self::Error> {
        if s.is_empty() || !s.chars().all(|c| c.is_ascii_alphanumeric() || "-!#$%&'*+.^_`|~".contains(c)) {
            return Err(HttpError::HeaderError(format!("Invalid Header Name Token: {}", s)));
        }
        Ok(Self(s))
    }
}

impl TryFrom<String> for HeaderName {
    type Error = HttpError;
    fn try_from(s: String) -> Result<Self, Self::Error> { Self::try_from(Rc::from(s)) }
}

impl TryFrom<&str> for HeaderName {
    type Error = HttpError;
    fn try_from(s: &str) -> Result<Self, Self::Error> { Self::try_from(Rc::from(s)) }
}

impl AsRef<str> for HeaderName { fn as_ref(&self) -> &str { &self.0 } }
impl std::fmt::Display for HeaderName { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) } }

/// Validated Header Value (RFC 7230 visible ASCII).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HeaderValue(Rc<str>);

impl TryFrom<Rc<str>> for HeaderValue {
    type Error = HttpError;
    fn try_from(s: Rc<str>) -> Result<Self, Self::Error> {
        if s.chars().any(|c| (c as u32) < 32 && c != '\t') {
            return Err(HttpError::HeaderError(format!("Invalid Header Value: {}", s)));
        }
        Ok(Self(s))
    }
}

impl TryFrom<String> for HeaderValue {
    type Error = HttpError;
    fn try_from(s: String) -> Result<Self, Self::Error> { Self::try_from(Rc::from(s)) }
}

impl TryFrom<&str> for HeaderValue {
    type Error = HttpError;
    fn try_from(s: &str) -> Result<Self, Self::Error> { Self::try_from(Rc::from(s)) }
}

impl AsRef<str> for HeaderValue { fn as_ref(&self) -> &str { &self.0 } }
impl std::fmt::Display for HeaderValue { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) } }

/// An Atomic HTTP Header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    name: HeaderName,
    value: HeaderValue,
}

impl Header {
    /// Pure Smart Constructor for Header.
    pub fn try_new(n: impl TryInto<HeaderName, Error=HttpError>, v: impl TryInto<HeaderValue, Error=HttpError>) -> Result<Self, HttpError> {
        Ok(Self {
            name: n.try_into()?,
            value: v.try_into()?,
        })
    }
    
    /// Returns the header name.
    pub fn name(&self) -> &HeaderName {
        &self.name
    }

    /// Returns the header value.
    pub fn value(&self) -> &HeaderValue {
        &self.value
    }
}

impl TryFrom<&str> for Header {
    type Error = HttpError;
    /// Constructs a Header from a "Name: Value" string.
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = s.splitn(2, ':').map(|p| p.trim()).collect();
        if parts.len() != 2 {
            return Err(HttpError::HeaderError(format!("Malformed Header Line: {}", s)));
        }
        Self::try_new(parts[0], parts[1])
    }
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

/// Formal Specification for Financial Grade Security Headers.
pub fn validate_security_headers(headers: &[Header]) -> Result<(), HttpError> {
    let check = |name: &str| headers.iter().any(|h| h.name().as_ref().eq_ignore_ascii_case(name));

    if !check("Strict-Transport-Security") {
        return Err(HttpError::ResponseError("Security Violation: Missing HSTS".into()));
    }
    
    let nosniff = headers.iter().find(|h| h.name().as_ref().eq_ignore_ascii_case("X-Content-Type-Options"));
    match nosniff {
        Some(h) if h.value().as_ref().eq_ignore_ascii_case("nosniff") => (),
        _ => return Err(HttpError::ResponseError("Security Violation: Missing/Invalid Nosniff".into())),
    }

    if !check("Content-Security-Policy") {
        return Err(HttpError::ResponseError("Security Violation: Missing CSP".into()));
    }

    Ok(())
}
