pub mod indent;
pub mod literal;
pub mod scan;
pub mod symbol;
pub mod token;

pub use token::{FPart, Op, TokKind, Token};

use crate::diag::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

/// Turn source text into a flat token stream (Indent/Dedent/Newline included).
pub fn scan(src: &str) -> Result<Vec<Token>, LexError> {
    scan::scan_tokens(src)
}
