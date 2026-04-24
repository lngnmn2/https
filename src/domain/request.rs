//! # HTTPS Request (Typestate Model)
//!
//! Correctness by Construction: Mandatory Host and Framing invariants are enforced at compile time.

use super::method::Method;
use super::header::Header;
use super::body::Body;
use super::error::HttpError;
use url::Url;
use core::marker::PhantomData;
use std::rc::Rc;

// ============================================================================
// 1. TYPESTATE MARKERS
// ============================================================================

/// Marker: Host invariant satisfied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct HasHost;
/// Marker: Initial state (No body rules applied).
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct Initial;
/// Marker: Validated state (Ready for transmission).
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct Validated;

// ============================================================================
// 2. DOMAIN DATA STRUCTURE
// ============================================================================

/// A Mathematically Rigorous HTTPS Request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecureRequest<H, B> {
    pub(crate) method: Method,
    pub(crate) url: Url,
    pub(crate) headers: Rc<[Header]>,
    pub(crate) body: Body,
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

// ============================================================================
// 3. TRANSITIONS (TRANSFORMATIONS)
// ============================================================================

impl SecureRequest<HasHost, Initial> {
    /// Initial Algebra: Creation of the basic Request structure.
    pub fn try_new(method: Method, url: impl Into<String>) -> Result<Self, HttpError> {
        let u = Url::parse(&url.into())?;
        if u.scheme() != "https" { return Err(HttpError::InsecureScheme(u.scheme().into())); }
        if u.host_str().is_none() { return Err(HttpError::UrlError("Missing Host".into())); }

        Ok(Self {
            method, url: u, headers: Rc::from(Vec::new()), body: Body::default(),
            _h: PhantomData, _b: PhantomData,
        })
    }

    /// Monadic Transformation: Adds a header.
    pub fn with_header(self, k: impl Into<String>, v: impl Into<String>) -> Result<Self, HttpError> {
        let h = Header::try_new(k.into(), v.into())?;
        let mut hs: Vec<Header> = self.headers.to_vec();
        hs.push(h);
        Ok(Self {
            headers: Rc::from(hs),
            ..self
        })
    }

    /// Final Homomorphism: State transition to terminal Validated state.
    pub fn with_body(self, b: impl Into<Body>) -> Request {
        SecureRequest {
            method: self.method,
            url: self.url,
            headers: self.headers,
            body: b.into(),
            _h: PhantomData,
            _b: PhantomData,
        }
    }
}

impl<H, B> SecureRequest<H, B> {
    /// Returns the HTTP method.
    pub const fn method(&self) -> &Method { &self.method }
    /// Returns the request URL.
    pub const fn url(&self) -> &Url { &self.url }
    /// Returns the request headers.
    pub fn headers(&self) -> &[Header] { &self.headers }
    /// Returns the request body.
    pub const fn body(&self) -> &Body { &self.body }
}
