//! IR optimiser over the lowered AST (owned by task 32).

pub mod dce;
pub mod fold;

use crate::ast::item::Item;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct OptReport {
    pub folded: usize,
    pub removed: usize,
}

/// Apply folding and dead-code elimination to a fixed point.
pub fn run(_items: &mut [Item]) -> OptReport {
    OptReport::default()
}
