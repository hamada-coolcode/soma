//! Structural invariants for parsed ASTs (owned by task 25).

use crate::diag::Diagnostic;
use super::item::Item;

/// Check spans, names, duplicate params/fields and empty bodies.
pub fn validate(_items: &[Item], _src_len: usize) -> Vec<Diagnostic> {
    Vec::new()
}
