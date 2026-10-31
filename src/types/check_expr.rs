use crate::ast::expr::Expr;
use crate::diag::Diagnostic;

use super::ty::{TypeId, TypeTable};

/// Infer the type of an expression (owned by task 11).
pub fn check(table: &mut TypeTable, _expr: &Expr) -> Result<TypeId, Diagnostic> {
    Ok(table.intern(crate::types::Type::Never))
}
