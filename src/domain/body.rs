//! # HTTP Body Domain Model
//!
//! Ref: https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages

use std::rc::Rc;

/// Newtype for HTTP Body to avoid naked types.
/// Uses Rc<[u8]> for efficient, immutable sharing of byte slices.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Body(Rc<[u8]>);

impl From<Vec<u8>> for Body {
    fn from(v: Vec<u8>) -> Self {
        Self(Rc::from(v))
    }
}

impl From<&[u8]> for Body {
    fn from(v: &[u8]) -> Self {
        Self(Rc::from(v))
    }
}

impl AsRef<[u8]> for Body {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Body {
    /// Returns the length of the body in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    /// Returns true if the body is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
