pub mod blocks;
pub mod expr;
pub mod signature;
pub mod types;
pub mod util;

use crate::diag::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
    pub expected: Vec<String>,
}

impl ParseError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
            expected: Vec::new(),
        }
    }
}

pub use blocks::parse_file;
pub use signature::Signature;
