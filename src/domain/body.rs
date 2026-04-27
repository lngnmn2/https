//! # HTTP Body Domain Model
//!
//! Ref: https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages

use std::rc::Rc;
use std::ops::Deref;

/// Newtype for HTTP Body to avoid naked types.
/// Uses Rc<[u8]> for efficient, immutable shared persistence.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Body(Rc<[u8]>);

impl Deref for Body {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<Vec<u8>> for Body { fn from(v: Vec<u8>) -> Self { Self(Rc::from(v)) } }
impl From<&[u8]> for Body { fn from(v: &[u8]) -> Self { Self(Rc::from(v)) } }
impl AsRef<[u8]> for Body { fn as_ref(&self) -> &[u8] { self.0.as_ref() } }

impl Body {
    /// Returns the length of the body in bytes.
    pub fn len(&self) -> usize { self.0.len() }
    /// Returns true if the body is empty.
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
}
