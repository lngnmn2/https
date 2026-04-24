//! # Protocol Interpreter (DSL Transducer)
//!
//! Homomorphism: Request Typestate -> STG Expr Initial Algebra.

use crate::domain::request::SecureRequest;
use crate::domain::response::Response;
use crate::domain::error::HttpError;
use crate::domain::status::Status;
use crate::domain::header::{Header, validate_security_headers};
use crate::domain::body::Body;
use crate::domain::host::Host;
use crate::domain::port::Port;
use crate::interpreter::stg::{Expr, Atom};
use std::io::{BufRead, BufReader, Read};
use std::rc::Rc;

// ----------------------------------------------------------------------------
// 1. DSL PLANNING (The Homomorphism)
// ----------------------------------------------------------------------------

/// Transforms a validated Request into a rigorous STG execution plan.
/// 
/// # Errors
/// Returns `HttpError` if URL parsing or formatting fails.
pub fn plan<H, B>(req: &SecureRequest<H, B>) -> Result<Expr<Response>, HttpError> {
    let host: Host = req.url().host_str().ok_or(HttpError::UrlError("Empty Host".into()))?.try_into()?;
    let port: Port = req.url().port_or_known_default().unwrap_or(443).into();
    let data = format_request(req)?;
    
    // Algebraic Chain: Case(Connect, \_ -> Case(Handshake, \_ -> Case(Write, \_ -> Case(Read, \res -> Pure(res)))))
    let h2 = host.clone();
    Ok(Expr::Case(
        Box::new(Expr::OpConnect(host, port)),
        Box::new(move |_| Expr::Case(
            Box::new(Expr::OpHandshake(h2)),
            Box::new(move |_| Expr::Case(
                Box::new(Expr::OpWrite(data)),
                Box::new(move |_| Expr::Case(
                    Box::new(Expr::OpRead),
                    Box::new(|res| Expr::Pure(Atom::Lit(Rc::new(res))))
                ))
            ))
        ))
    ))
}

// ----------------------------------------------------------------------------
// 2. WIRE SERIALIZATION (Pure Expression-Oriented)
// ----------------------------------------------------------------------------

/// Formats a Request into its wire representation as bytes.
/// 
/// # Errors
/// Returns `HttpError` if any internal formatting invariants are violated.
pub fn format_request<H, B>(req: &SecureRequest<H, B>) -> Result<Vec<u8>, HttpError> {
    let request_line = format!("{} {} HTTP/1.1\r\n", 
        req.method(), 
        req.url().path().is_empty().then_some("/").unwrap_or(req.url().path())
    );

    let mandatory_headers = [
        (!req.headers().iter().any(|h| h.name().as_ref().eq_ignore_ascii_case("Host")))
            .then(|| format!("Host: {}\r\n", req.url().host_str().unwrap_or(""))),
        (!req.headers().iter().any(|h| h.name().as_ref().eq_ignore_ascii_case("User-Agent")))
            .then(|| "User-Agent: SecureFinancialClient/1.0\r\n".to_string()),
        Some("Connection: close\r\n".to_string()),
        (!req.body().is_empty())
            .then(|| format!("Content-Length: {}\r\n", req.body().len())),
    ].into_iter().flatten().collect::<String>();

    let custom_headers = req.headers().iter()
        .map(|h| format!("{}: {}\r\n", h.name(), h.value()))
        .collect::<String>();

    let head = [request_line, mandatory_headers, custom_headers, "\r\n".to_string()].concat().into_bytes();
    
    Ok([head, req.body().as_ref().to_vec()].concat())
}

// ----------------------------------------------------------------------------
// 3. RESPONSE PARSING (Monadic FSM)
// ----------------------------------------------------------------------------

const MAX_LINE: usize = 8192;
const MAX_BODY: usize = 20 * 1024 * 1024;

/// Reads and parses an HTTP response from the given reader.
/// 
/// # Errors
/// Returns `HttpError` if parsing fails or security invariants are violated.
pub fn read_response<R: Read>(reader: R) -> Result<Response, HttpError> {
    let mut br = BufReader::new(reader);
    
    // Sequential State Transitions (Monadic Chain)
    let status = parse_status(&read_line(&mut br)?)?;
    let headers = read_headers(&mut br)?;
    
    // RFC 7230 Invariants Validation
    validate_invariants(&headers)?;
    validate_security_headers(&headers)?;
    
    let body = read_body(&mut br, &headers)?;
    Ok(Response::new(status, headers, body))
}

fn read_line<R: BufRead>(r: &mut R) -> Result<String, HttpError> {
    let mut l = String::new();
    let n = r.take(MAX_LINE as u64).read_line(&mut l)?;
    if n == 0 { return Err(HttpError::ResponseError("EOF".into())); }
    Ok(l.trim().to_string())
}

fn parse_status(l: &str) -> Result<Status, HttpError> {
    let p: Vec<&str> = l.splitn(3, ' ').collect();
    if p.len() < 2 || p[0] != "HTTP/1.1" { return Err(HttpError::ResponseError(l.into())); }
    p[1].parse::<u16>().map(Status::from).map_err(|_| HttpError::ResponseError(p[1].into()))
}

fn read_headers<R: BufRead>(r: &mut R) -> Result<Rc<[Header]>, HttpError> {
    std::iter::from_fn(|| Some(read_line(r)))
        .take_while(|res| res.as_ref().map_or(true, |l| !l.is_empty()))
        .map(|res| res.and_then(|l| Header::try_from(l.as_str())))
        .collect::<Result<Vec<_>, _>>()
        .map(Rc::from)
}

fn validate_invariants(hs: &[Header]) -> Result<(), HttpError> {
    let has_cl = hs.iter().any(|h| h.name().as_ref().eq_ignore_ascii_case("Content-Length"));
    let has_te = hs.iter().any(|h| h.name().as_ref().eq_ignore_ascii_case("Transfer-Encoding"));
    if has_cl && has_te { return Err(HttpError::ResponseError("Conflicting Framing".into())); }
    Ok(())
}

fn read_body<R: BufRead>(r: &mut R, hs: &[Header]) -> Result<Body, HttpError> {
    let cl = hs.iter().find(|h| h.name().as_ref().eq_ignore_ascii_case("Content-Length"))
        .and_then(|h| h.value().as_ref().parse::<usize>().ok());
        
    if let Some(len) = cl {
        if len > MAX_BODY { return Err(HttpError::ResponseError("Body too large".into())); }
        let mut b = vec![0u8; len];
        r.read_exact(&mut b)?;
        Ok(Body::from(b))
    } else {
        let mut b = Vec::new();
        r.take(MAX_BODY as u64).read_to_end(&mut b)?;
        Ok(Body::from(b))
    }
}
