import sys

with open("src/types/check_expr.rs", "w") as f:
    f.write("""use crate::ast::expr::Expr;
use crate::diag::{Diagnostic, ErrorKind, Span};

use super::ty::{TypeId, TypeTable};

/// Infer the type of an expression (owned by task 11).
pub fn check(table: &mut TypeTable, _expr: &Expr) -> Result<TypeId, Diagnostic> {
    Ok(table.intern(crate::types::Type::Never))
}
""")
