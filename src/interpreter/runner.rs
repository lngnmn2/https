//! # High-level Runner
//!
//! Pure functional runner entry point.

use crate::domain::{SecureRequest, Response, HttpError};
use super::protocol;
use super::stg::StgMachine;
use std::rc::Rc;

/// Executes a planned interaction to completion.
pub fn execute<H, B>(req: &SecureRequest<H, B>) -> Result<Response, HttpError> {
    let interaction = protocol::plan(req)?;
    let machine = StgMachine::<Response>::new();
    let res_rc = machine.evaluate(interaction)?;
    
    // Final persistent extraction.
    Ok(Rc::try_unwrap(res_rc).unwrap_or_else(|r| (*r).clone()))
}
