//! Development interpreter for `soma run` (owned by task 15).
//!
//! This is NOT the zero-runtime-cost backend; it exists so semantics can be
//! tested without codegen.

pub mod eval;
pub mod value;

pub use value::Value;

use crate::diag::{Diagnostic, ErrorKind, Span};

#[derive(Debug, Default)]
pub struct RunOutput {
    pub stdout: String,
}

pub fn run_source(_name: &str, _src: &str) -> Result<RunOutput, Diagnostic> {
    Err(Diagnostic::new(
        ErrorKind::Other,
        "interpreter not implemented",
        Span::default(),
    ))
}

pub fn run_file(path: &str) -> Result<RunOutput, Diagnostic> {
    let src = std::fs::read_to_string(path).map_err(|e| {
        Diagnostic::new(ErrorKind::Io, format!("{path}: {e}"), Span::default())
    })?;
    run_source(path, &src)
}
