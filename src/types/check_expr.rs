use crate::ast::expr::Expr;
use crate::diag::{Diagnostic, ErrorKind, Span};

use super::ty::{TypeId, TypeTable};

/// Infer the type of an expression (owned by task 11).
pub fn check(_table: &mut TypeTable, _expr: &Expr) -> Result<TypeId, Diagnostic> {
    Err(Diagnostic::new(
        ErrorKind::Type,
        "type checking not implemented",
        Span::default(),
    ))
}
