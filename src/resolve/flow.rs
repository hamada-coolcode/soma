//! Definite-assignment / use-before-assign analysis (owned by task 19).

use crate::ast::item::Item;
use crate::diag::Diagnostic;

pub fn check(_items: &[Item]) -> Vec<Diagnostic> {
    Vec::new()
}
