//! # HTTPS Response Domain Model
//!
//! Ref: https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages

use super::header::Header;
use super::status::Status;
use super::body::Body;
use super::error::HttpError;
use crate::interpreter::protocol;
use std::io::Cursor;
use std::rc::Rc;

/// An HTTP Response.
///
/// Encapsulates the status, headers, and body received from an HTTPS server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub(crate) status: Status,
    pub(crate) headers: Rc<[Header]>,
    pub(crate) body: Body,
}

impl Response {
    /// Creates a new Response from its constituent parts.
    pub fn new(status: Status, headers: impl Into<Rc<[Header]>>, body: Body) -> Self {
        Self {
            status,
            headers: headers.into(),
            body,
        }
    }

    /// Returns the HTTP status.
    pub fn status(&self) -> Status {
        self.status
    }

    /// Returns the response headers.
    pub fn headers(&self) -> &[Header] {
        &self.headers
    }

    /// Returns the response body.
    pub fn body(&self) -> &Body {
        &self.body
    }
}

impl TryFrom<&[u8]> for Response {
    type Error = HttpError;

    /// Natural Transformation from bytes to Response.
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = Cursor::new(bytes);
        protocol::read_response(&mut cursor)
    }
}

impl TryFrom<Vec<u8>> for Response {
    type Error = HttpError;

    /// Natural Transformation from Vec<u8> to Response.
    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(bytes.as_slice())
    }
}
