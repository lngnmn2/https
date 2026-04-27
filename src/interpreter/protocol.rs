//! # Protocol Interpreter (DSL Transducer)
//!
//! Pure functional protocol transformations.

use crate::domain::{SecureRequest, Response, HttpError, Status, Header, Body, Port, SecurityLevel};
use crate::interpreter::stg::{Expr, Atom};
use std::io::{BufRead, BufReader, Read};
use std::rc::Rc;

// --- RESOURCE LIMITS ---
const MAX_LINE: usize = 8192;
const MAX_BODY: usize = 20 * 1024 * 1024;

// ----------------------------------------------------------------------------
// 1. DSL PLANNING (The Homomorphism)
// ----------------------------------------------------------------------------

/// Transforms a validated Request into a rigorous STG execution plan.
pub fn plan<H, B>(req: &SecureRequest<H, B>) -> Result<Expr<Response>, HttpError> {
    let host: crate::domain::Host = req.url().host_str()
        .ok_or(HttpError::UrlError(Rc::from("Empty Host")))?
        .try_into()?;
    let port: Port = req.url().port_or_known_default().unwrap_or(443).into();
    let data = format_request(req)?;
    let level = req.security_level();
    
    let h2 = host.clone();
    Ok(Expr::Case(
        Box::new(Expr::OpConnect(host, port)),
        Box::new(move |_| Expr::Case(
            Box::new(Expr::OpHandshake(h2)),
            Box::new(move |_| Expr::Case(
                Box::new(Expr::OpWrite(Rc::from(data))),
                Box::new(move |_| Expr::Case(
                    Box::new(Expr::OpRead(level)),
                    Box::new(|res| Expr::Pure(Atom::Lit(Rc::new(res))))
                ))
            ))
        ))
    ))
}

// ----------------------------------------------------------------------------
// 2. WIRE SERIALIZATION (Pure Transformation)
// ----------------------------------------------------------------------------

/// Formats a Request into its wire representation as a pure transformation.
pub fn format_request<H, B>(req: &SecureRequest<H, B>) -> Result<Vec<u8>, HttpError> {
    let line = format!("{} {} HTTP/1.1\r\n", req.method().as_str(), req.url().path());
    let mandatory = [
        (!req.headers().iter().any(|h| h.name().as_ref().eq_ignore_ascii_case("Host")))
            .then(|| format!("Host: {}\r\n", req.url().host_str().unwrap_or(""))),
        Some("Connection: close\r\n".to_string()),
        (!req.body().is_empty()).then(|| format!("Content-Length: {}\r\n", req.body().len())),
    ].into_iter().flatten().collect::<String>();

    let custom = req.headers().iter().map(|h| format!("{}: {}\r\n", h.name(), h.value())).collect::<String>();
    
    Ok([line, mandatory, custom, "\r\n".into()].concat()
        .into_bytes()
        .into_iter()
        .chain(req.body().as_ref().to_vec())
        .collect())
}

// ----------------------------------------------------------------------------
// 3. EFFECTFUL PARSING (Imperative Shell Boundary)
// ----------------------------------------------------------------------------

/// Reads and parses an HTTP response from a raw reader.
pub fn read_response_pure<R: Read>(reader: R, level: SecurityLevel) -> Result<(R, Response), HttpError> {
    // Note: BufReader and read_line necessitate local mutation at the I/O boundary.
    // We isolate this within this specific infrastructure adapter.
    let mut br = BufReader::new(reader);
    let status = parse_status(&read_line_internal(&mut br)?)?;
    let headers = read_headers_internal(&mut br)?;
    
    validate_protocol_invariants(&headers)
        .and_then(|_| crate::domain::header::validate_security_headers(&headers, level))
        .and_then(|_| read_body_internal(&mut br, &headers))
        .map(|body| (br.into_inner(), Response::new(status, headers, body)))
}

fn read_line_internal<R: BufRead>(r: &mut R) -> Result<String, HttpError> {
    let mut l = String::new();
    let n = r.take(MAX_LINE as u64).read_line(&mut l).map_err(|e| HttpError::TransportError(Rc::from(e.to_string())))?;
    if n == 0 { return Err(HttpError::ResponseError(Rc::from("EOF"))); }
    Ok(l.trim().into())
}

fn parse_status(l: &str) -> Result<Status, HttpError> {
    let p: Vec<&str> = l.splitn(3, ' ').collect();
    if p.len() < 2 || p[0] != "HTTP/1.1" { return Err(HttpError::ResponseError(Rc::from(l))); }
    p[1].parse::<u16>().map(Status::from).map_err(|_| HttpError::ResponseError(Rc::from(p[1])))
}

fn read_headers_internal<R: BufRead>(r: &mut R) -> Result<Rc<[Header]>, HttpError> {
    std::iter::from_fn(|| Some(read_line_internal(r)))
        .take_while(|res| res.as_ref().map_or(true, |l| !l.is_empty()))
        .map(|res| res.and_then(|l| Header::try_from(l.as_str())))
        .collect::<Result<Vec<_>, _>>()
        .map(Rc::from)
}

fn validate_protocol_invariants(hs: &[Header]) -> Result<(), HttpError> {
    let has_cl = hs.iter().any(|h| h.name().as_ref().eq_ignore_ascii_case("Content-Length"));
    let has_te = hs.iter().any(|h| h.name().as_ref().eq_ignore_ascii_case("Transfer-Encoding"));
    if has_cl && has_te { 
        Err(HttpError::ResponseError(Rc::from("Conflicting Framing: CL and TE present"))) 
    } else { 
        Ok(()) 
    }
}

fn read_body_internal<R: BufRead>(r: &mut R, hs: &[Header]) -> Result<Body, HttpError> {
    let cl = hs.iter().find(|h| h.name().as_ref().eq_ignore_ascii_case("Content-Length"))
        .and_then(|h| h.value().as_ref().parse::<usize>().ok());
    
    match cl {
        Some(len) if len <= MAX_BODY => {
            let mut b = vec![0u8; len];
            r.read_exact(&mut b).map_err(|e| HttpError::TransportError(Rc::from(e.to_string())))?;
            Ok(Body::from(b))
        }
        None => {
            let mut b = Vec::new();
            r.take(MAX_BODY as u64).read_to_end(&mut b).map_err(|e| HttpError::TransportError(Rc::from(e.to_string())))?;
            Ok(Body::from(b))
        }
        _ => Err(HttpError::ResponseError(Rc::from("Too Large")))
    }
}
