//! # High-level Runner
//!
//! Pure functional runner entry point.

use crate::domain::{SecureRequest, Response, HttpError};
use super::protocol;
use super::stg::StgMachine;
use std::rc::Rc;

/// Executes a planned interaction to completion.
pub fn execute<H, B>(req: &SecureRequest<H, B>) -> Result<Response, HttpError> {
    protocol::plan(req)?
        .let_eval(StgMachine::<Response>::new())?
        .try_extract()
}

impl<A: 'static> crate::interpreter::stg::Expr<A> {
    fn let_eval(self, machine: StgMachine<A>) -> Result<Rc<A>, HttpError> {
        machine.evaluate(self)
    }
}

trait Extract<T> {
    fn try_extract(self) -> Result<T, HttpError>;
}

impl Extract<Response> for Rc<Response> {
    fn try_extract(self) -> Result<Response, HttpError> {
        Ok(Rc::try_unwrap(self).unwrap_or_else(|r| (*r).clone()))
    }
}
