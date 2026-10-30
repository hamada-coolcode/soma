//! `soma test` — execute `test "…"` blocks (owned by task 24).

use crate::diag::{Diagnostic, ErrorKind, Span};

#[derive(Debug, Default)]
pub struct TestReport {
    pub passed: usize,
    pub failed: usize,
    pub output: String,
}

pub fn run_tests(_name: &str, _src: &str) -> Result<TestReport, Diagnostic> {
    Err(Diagnostic::new(
        ErrorKind::Other,
        "test runner not implemented",
        Span::default(),
    ))
}
