//! # HTTPS Response Domain Model
//!
//! Ref: https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages

use super::header::{Header, SecurityLevel};
use super::status::Status;
use super::body::Body;
use super::error::HttpError;
use crate::interpreter::protocol;
use std::io::Cursor;
use std::rc::Rc;

/// An HTTP Response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub(crate) status: Status,
    pub(crate) headers: Rc<[Header]>,
    pub(crate) body: Body,
}

impl Response {
    /// Creates a new Response.
    pub fn new(status: Status, headers: impl Into<Rc<[Header]>>, body: Body) -> Self {
        Self { status, headers: headers.into(), body }
    }
    /// Returns the status.
    pub const fn status(&self) -> Status { self.status }
    /// Returns the headers.
    pub fn headers(&self) -> &[Header] { &self.headers }
    /// Returns the body.
    pub const fn body(&self) -> &Body { &self.body }
}

impl TryFrom<&[u8]> for Response {
    type Error = HttpError;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let cursor = Cursor::new(bytes);
        // Note: For raw byte parsing, we assume Standard security level as we may 
        // not have the full request context (which contains the intended policy).
        protocol::read_response_pure(cursor, SecurityLevel::Standard).map(|(_, r)| r)
    }
}

impl TryFrom<Vec<u8>> for Response {
    type Error = HttpError;
    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> { Self::try_from(bytes.as_slice()) }
}
