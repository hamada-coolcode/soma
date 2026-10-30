//! C backend — the documented stopgap before LLVM (owned by task 26).

pub mod c;

use crate::ast::item::Item;

/// Transpile lowered items to a single self-contained C translation unit.
pub fn emit_c(items: &[Item]) -> String {
    c::emit(items)
}
