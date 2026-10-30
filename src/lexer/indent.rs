use crate::diag::Span;

use super::{LexError, Token};

/// Tracks the indentation stack for a file and emits Indent/Dedent tokens.
///
/// Contract (see jules/grammar.ebnf):
/// - widths are compared by value, not by column count arithmetic
/// - a deeper line emits one Indent
/// - a shallower line emits one Dedent per level popped
/// - a width that matches no open level is an error
/// - tabs are rejected
#[derive(Debug, Default)]
pub struct IndentTracker {
    /// Open indentation widths; starts with 0 for the file body.
    pub levels: Vec<usize>,
}

impl IndentTracker {
    pub fn new() -> Self {
        Self { levels: vec![0] }
    }

    /// Feed the measured indentation of a new line. Returns the
    /// Indent/Dedent tokens that must be emitted before the line's tokens.
    pub fn feed(&mut self, _width: usize, _span: Span) -> Result<Vec<Token>, LexError> {
        Ok(Vec::new())
    }

    /// Emit the Dedents needed at end of file.
    pub fn finish(&mut self) -> Vec<Token> {
        Vec::new()
    }
}
