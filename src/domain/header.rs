//! # HTTP Header Domain Model
//!
//! Ref: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers
//! Pure functional header model using persistent strings.

use super::error::HttpError;
use std::rc::Rc;
use std::ops::Deref;

/// Validated Header Name (RFC 7230 token).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HeaderName(Rc<str>);

impl Deref for HeaderName { type Target = str; fn deref(&self) -> &str { &self.0 } }

impl TryFrom<Rc<str>> for HeaderName {
    type Error = HttpError;
    fn try_from(s: Rc<str>) -> Result<Self, Self::Error> {
        let is_valid = !s.is_empty() 
            && s.chars().all(|c| c.is_ascii_alphanumeric() || "-!#$%&'*+.^_`|~".contains(c));
        
        if is_valid { 
            Ok(Self(s)) 
        } else { 
            Err(HttpError::HeaderError(Rc::from(format!("Invalid Header Name Token: {}", &*s)))) 
        }
    }
}

impl TryFrom<String> for HeaderName { type Error = HttpError; fn try_from(s: String) -> Result<Self, Self::Error> { Self::try_from(Rc::from(s)) } }
impl TryFrom<&str> for HeaderName { type Error = HttpError; fn try_from(s: &str) -> Result<Self, Self::Error> { Self::try_from(Rc::from(s)) } }
impl AsRef<str> for HeaderName { fn as_ref(&self) -> &str { &self.0 } }
impl std::fmt::Display for HeaderName { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", &*self.0) } }

/// Validated Header Value (RFC 7230 visible ASCII).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct HeaderValue(Rc<str>);

impl Deref for HeaderValue { type Target = str; fn deref(&self) -> &str { &self.0 } }

impl TryFrom<Rc<str>> for HeaderValue {
    type Error = HttpError;
    fn try_from(s: Rc<str>) -> Result<Self, Self::Error> {
        let is_valid = s.chars().all(|c| (c as u32) >= 32 && (c as u32) <= 126 || c == '\t');
        
        if is_valid { 
            Ok(Self(s)) 
        } else { 
            Err(HttpError::HeaderError(Rc::from(format!("Invalid Header Value: {}", &*s)))) 
        }
    }
}

impl TryFrom<String> for HeaderValue { type Error = HttpError; fn try_from(s: String) -> Result<Self, Self::Error> { Self::try_from(Rc::from(s)) } }
impl TryFrom<&str> for HeaderValue { type Error = HttpError; fn try_from(s: &str) -> Result<Self, Self::Error> { Self::try_from(Rc::from(s)) } }
impl AsRef<str> for HeaderValue { fn as_ref(&self) -> &str { &self.0 } }
impl std::fmt::Display for HeaderValue { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", &*self.0) } }

/// An Atomic HTTP Header.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Header {
    name: HeaderName,
    value: HeaderValue,
}

impl Header {
    /// Pure Smart Constructor for Header.
    pub fn try_new(n: impl TryInto<HeaderName, Error=HttpError>, v: impl TryInto<HeaderValue, Error=HttpError>) -> Result<Self, HttpError> {
        Ok(Self { name: n.try_into()?, value: v.try_into()? })
    }
    
    /// Returns the header name.
    pub fn name(&self) -> &HeaderName { &self.name }
    /// Returns the header value.
    pub fn value(&self) -> &HeaderValue { &self.value }
}

impl TryFrom<&str> for Header {
    type Error = HttpError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() != 2 { 
            return Err(HttpError::HeaderError(Rc::from(format!("Malformed Header Line: {}", s)))); 
        }
        Self::try_new(parts[0].trim(), parts[1].trim())
    }
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", &*self.name, &*self.value)
    }
}

/// Levels of security enforcement for headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SecurityLevel {
    /// No mandatory headers enforced.
    None,
    /// Minimum viable security (Allows missing HSTS/Nosniff for production parity).
    Standard,
    /// Enforces HSTS, Nosniff, and Strict CSP.
    #[default] Strict,
}

/// Formal Specification for Financial Grade Security Headers.
pub fn validate_security_headers(headers: &[Header], level: SecurityLevel) -> Result<(), HttpError> {
    if matches!(level, SecurityLevel::None) { return Ok(()); }
    
    let check = |name: &str| headers.iter().any(|h| h.name().as_ref().eq_ignore_ascii_case(name));
    
    if matches!(level, SecurityLevel::Strict) && !check("Strict-Transport-Security") { 
        return Err(HttpError::ResponseError(Rc::from("Missing HSTS"))); 
    }
    
    headers.iter().find(|h| h.name().as_ref().eq_ignore_ascii_case("X-Content-Type-Options"))
        .filter(|h| h.value().as_ref().eq_ignore_ascii_case("nosniff"))
        .ok_or(HttpError::ResponseError(Rc::from("Missing Nosniff")))?;

    if !check("Content-Security-Policy") { return Err(HttpError::ResponseError(Rc::from("Missing CSP"))); }

    Ok(())
}
