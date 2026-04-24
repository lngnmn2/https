//! # High-level Runner
//!
//! Imperative Shell entry point for protocol execution.

use crate::domain::request::SecureRequest;
use crate::domain::response::Response;
use crate::domain::error::HttpError;
use super::protocol;
use super::stg::StgMachine;
use std::rc::Rc;

/// Executes a planned interaction to completion.
///
/// Natural Transformation: Expr<Response> -> Result<Response, HttpError>
pub fn execute<H, B>(req: &SecureRequest<H, B>) -> Result<Response, HttpError> {
    let interaction = protocol::plan(req)?;
    let mut machine = StgMachine::<Response>::new();
    let rc_res = machine.evaluate(interaction)?;
    
    // Attempt to unwrap Rc if unique, otherwise clone the response structure
    Ok(Rc::try_unwrap(rc_res).unwrap_or_else(|r| (*r).clone()))
}
