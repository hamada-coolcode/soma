//! Machine-readable AST dump (owned by task 22).

use crate::ast::item::Item;

/// Deterministic pretty-printed JSON for a parsed file.
pub fn emit_json(_items: &[Item]) -> String {
    "[]".to_string()
}
