//! # HTTPS Request (Typestate Model)
//!
//! Pure functional request construction using a declarative Typestate DSL.

use super::method::Method;
use super::header::{Header, SecurityLevel};
use super::body::Body;
use super::error::HttpError;
use url::Url;
use core::marker::PhantomData;
use std::rc::Rc;

// --- TYPESTATE MARKERS ---
/// Marker: Host invariant satisfied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct HasHost;
/// Marker: Initial state (No body rules applied).
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct Initial;
/// Marker: Validated state (Ready for transmission).
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct Validated;

/// A Mathematically Rigorous HTTPS Request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecureRequest<H, B> {
    pub(crate) method: Method,
    pub(crate) url: Url,
    pub(crate) headers: Rc<[Header]>,
    pub(crate) body: Body,
    pub(crate) security_level: SecurityLevel,
    pub(crate) _h: PhantomData<H>,
    pub(crate) _b: PhantomData<B>,
}

/// terminal state: Ready for execution.
pub type Request = SecureRequest<HasHost, Validated>;

/// Initial Algebra state.
pub type InitialRequest = SecureRequest<HasHost, Initial>;

impl TryFrom<(&str, &str)> for InitialRequest {
    type Error = HttpError;
    fn try_from((m, u): (&str, &str)) -> Result<Self, Self::Error> {
        SecureRequest::try_new(m.try_into()?, u)
    }
}

impl SecureRequest<HasHost, Initial> {
    /// Initial Algebra: Creation.
    pub fn try_new(method: Method, url: impl Into<Rc<str>>) -> Result<Self, HttpError> {
        let u = Url::parse(&*url.into())?;
        if u.scheme() != "https" { return Err(HttpError::InsecureScheme(Rc::from(u.scheme()))); }
        let _ = u.host_str().ok_or(HttpError::UrlError(Rc::from("Missing Host")))?;

        Ok(Self {
            method, url: u, headers: Rc::from([]), body: Body::default(),
            security_level: SecurityLevel::Strict,
            _h: PhantomData, _b: PhantomData,
        })
    }

    /// Monadic Transformation: Header addition.
    pub fn with_header(self, k: impl Into<Rc<str>>, v: impl Into<Rc<str>>) -> Result<Self, HttpError> {
        let h = Header::try_new(k.into(), v.into())?;
        Ok(Self {
            headers: self.headers.iter().cloned().chain(std::iter::once(h)).collect::<Rc<[Header]>>(),
            ..self
        })
    }

    /// Final Homomorphism: State transition to terminal Validated state.
    pub fn with_body(self, b: impl Into<Body>) -> Request {
        SecureRequest {
            method: self.method, url: self.url, headers: self.headers, body: b.into(),
            security_level: self.security_level,
            _h: PhantomData, _b: PhantomData,
        }
    }
    
    /// Shortcut for empty body terminal transition.
    pub fn build(self) -> Request {
        self.with_body(Body::default())
    }
}

impl<H, B> SecureRequest<H, B> {
    /// Configures the security level.
    pub fn with_security_level(self, security_level: SecurityLevel) -> Self {
        Self { security_level, ..self }
    }
    /// Returns the method.
    pub const fn method(&self) -> &Method { &self.method }
    /// Returns the URL.
    pub const fn url(&self) -> &Url { &self.url }
    /// Returns the headers.
    pub fn headers(&self) -> &[Header] { &self.headers }
    /// Returns the body.
    pub const fn body(&self) -> &Body { &self.body }
    /// Returns the security level.
    pub const fn security_level(&self) -> SecurityLevel { self.security_level }
}
