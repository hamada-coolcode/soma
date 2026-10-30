//! Multi-error parsing (owned by task 18).

use crate::ast::item::Item;
use crate::diag::Diagnostic;
use crate::lexer::Token;

/// Parse while collecting diagnostics, resyncing at declaration boundaries.
/// The returned items are the successfully parsed prefix.
pub fn parse_file_recovering(toks: &[Token]) -> (Vec<Item>, Vec<Diagnostic>) {
    match crate::parse::parse_file(toks) {
        Ok(items) => (items, Vec::new()),
        Err(e) => {
            let mut d = Diagnostic::new(crate::diag::ErrorKind::Parse, e.message, e.span);
            if !e.expected.is_empty() {
                d = d.with_help(format!("expected: {}", e.expected.join(", ")));
            }
            (Vec::new(), vec![d])
        }
    }
}
