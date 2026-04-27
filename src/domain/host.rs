//! # Host Domain Model
//!
//! Validated Hostname (RFC 1123) using persistent strings.

use super::error::HttpError;
use std::rc::Rc;
use std::ops::Deref;

/// Validated Hostname Newtype.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Host(Rc<str>);

impl Deref for Host {
    type Target = str;
    fn deref(&self) -> &str {
        self.0.as_ref()
    }
}

impl TryFrom<Rc<str>> for Host {
    type Error = HttpError;
    fn try_from(s: Rc<str>) -> Result<Self, Self::Error> {
        let is_valid = !s.is_empty() 
            && s.len() <= 253 
            && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
        
        if is_valid { 
            Ok(Self(s)) 
        } else { 
            Err(HttpError::UrlError(Rc::from(format!("Invalid Host: {}", s)))) 
        }
    }
}

impl TryFrom<String> for Host {
    type Error = HttpError;
    fn try_from(s: String) -> Result<Self, Self::Error> { Self::try_from(Rc::from(s)) }
}

impl TryFrom<&str> for Host {
    type Error = HttpError;
    fn try_from(s: &str) -> Result<Self, Self::Error> { Self::try_from(Rc::from(s)) }
}

impl AsRef<str> for Host { fn as_ref(&self) -> &str { self.0.as_ref() } }
impl std::fmt::Display for Host { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) } }
