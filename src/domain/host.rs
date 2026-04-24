//! # Host Domain Model
//!
//! Validated Hostname (RFC 1123).

use super::error::HttpError;
use std::rc::Rc;

/// Validated Hostname Newtype.
/// Uses Rc<str> for immutable persistence and efficient sharing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Host(Rc<str>);

impl TryFrom<String> for Host {
    type Error = HttpError;
    /// Constructs a Host from a String, enforcing RFC 1123 validation.
    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.is_empty() || s.len() > 253 || !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-') {
            return Err(HttpError::UrlError(format!("Invalid Host: {}", s)));
        }
        Ok(Self(Rc::from(s)))
    }
}

impl TryFrom<&str> for Host {
    type Error = HttpError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::try_from(s.to_string())
    }
}

impl AsRef<str> for Host {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
