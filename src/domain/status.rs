//! # HTTP Status Domain Model
//!
//! Ref: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status

use super::error::HttpError;

/// Validated HTTP response status codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    /// 200 OK.
    #[default] Ok, 
    /// 201 Created.
    Created, 
    /// 202 Accepted.
    Accepted, 
    /// 204 No Content.
    NoContent,
    /// 301 Moved Permanently.
    MovedPermanently, 
    /// 302 Found.
    Found,
    /// 400 Bad Request.
    BadRequest, 
    /// 401 Unauthorized.
    Unauthorized, 
    /// 403 Forbidden.
    Forbidden, 
    /// 404 Not Found.
    NotFound,
    /// 500 Internal Server Error.
    InternalServerError, 
    /// 501 Not Implemented.
    NotImplemented, 
    /// 502 Bad Gateway.
    BadGateway, 
    /// 503 Service Unavailable.
    ServiceUnavailable,
    /// Unknown status code.
    Unknown(u16),
}

impl Status {
    /// Returns the numerical status code.
    pub const fn code(&self) -> u16 {
        match self {
            Status::Ok => 200, Status::Created => 201, Status::Accepted => 202, Status::NoContent => 204,
            Status::MovedPermanently => 301, Status::Found => 302,
            Status::BadRequest => 400, Status::Unauthorized => 401, Status::Forbidden => 403, Status::NotFound => 404,
            Status::InternalServerError => 500, Status::NotImplemented => 501, Status::BadGateway => 502, Status::ServiceUnavailable => 503,
            Status::Unknown(c) => *c,
        }
    }

    /// Returns the standard reason phrase.
    pub const fn reason_phrase(&self) -> &'static str {
        match self {
            Status::Ok => "OK", Status::Created => "Created", Status::Accepted => "Accepted", Status::NoContent => "No Content",
            Status::MovedPermanently => "Moved Permanently", Status::Found => "Found",
            Status::BadRequest => "Bad Request", Status::Unauthorized => "Unauthorized", Status::Forbidden => "Forbidden", Status::NotFound => "Not Found",
            Status::InternalServerError => "Internal Server Error", Status::NotImplemented => "Not Implemented", Status::BadGateway => "Bad Gateway", Status::ServiceUnavailable => "Service Unavailable",
            Status::Unknown(_) => "Unknown",
        }
    }
}

impl From<u16> for Status {
    /// Constructs a Status from a numerical code.
    fn from(c: u16) -> Self {
        match c {
            200 => Status::Ok, 201 => Status::Created, 202 => Status::Accepted, 204 => Status::NoContent,
            301 => Status::MovedPermanently, 302 => Status::Found,
            400 => Status::BadRequest, 401 => Status::Unauthorized, 403 => Status::Forbidden, 404 => Status::NotFound,
            500 => Status::InternalServerError, 501 => Status::NotImplemented, 502 => Status::BadGateway, 503 => Status::ServiceUnavailable,
            _ => Status::Unknown(c),
        }
    }
}

impl TryFrom<&str> for Status {
    type Error = HttpError;
    /// Constructs a Status from a string slice representing a numerical code.
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse::<u16>().map(Status::from).map_err(|_| HttpError::ResponseError(format!("Invalid Status Code: {}", s)))
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.code(), self.reason_phrase())
    }
}
