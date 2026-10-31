//! Name resolution (owned by task 09).

pub mod scope;
pub mod symbol;

pub use scope::Scope;
pub use symbol::{Symbol, SymbolKind};

use crate::ast::item::Item;
use crate::diag::Diagnostic;

/// Resolve every name in the file and report undefined/immutable/duplicate errors.
pub fn check(_items: &[Item]) -> Vec<Diagnostic> {
    Vec::new()
}
