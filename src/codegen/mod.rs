//! C backend — the documented stopgap before LLVM (owned by task 26).

pub mod c;

use crate::ast::item::Item;
use crate::diag::Diagnostic;

/// Transpile lowered items to a single self-contained C translation unit.
pub fn emit_c(items: &[Item]) -> String {
    c::emit(items)
}

/// Transpile lowered items to a C translation unit, returning an error on unsupported constructs.
pub fn emit_c_checked(items: &[Item]) -> Result<String, Diagnostic> {
    c::emit_checked(items)
}
