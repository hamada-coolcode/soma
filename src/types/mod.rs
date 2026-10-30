//! Type checking (tasks 10-14).

pub mod check_expr;
pub mod check_item;
pub mod check_match;
pub mod generic;
pub mod ty;
pub mod unify;

pub use ty::{Type, TypeId, TypeTable};
pub use unify::{unify, UnifyError};

use crate::ast::item::Item;
use crate::diag::Diagnostic;

/// Whole-file type check.
pub fn check(items: &[Item]) -> Vec<Diagnostic> {
    check_item::check_items(items)
}
