//! Memory analysis: moves and ownership (owned by task 27).

use crate::ast::item::Item;
use crate::diag::Diagnostic;

/// Conservative move/ownership pass. Warnings and errors only; no rewrites.
pub fn check(_items: &[Item]) -> Vec<Diagnostic> {
    Vec::new()
}
