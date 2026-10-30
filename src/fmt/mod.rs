//! `soma fmt` — canonical source printer (owned by task 16).

pub mod print;

use crate::ast::item::Item;

/// Format a whole file. Must be idempotent: fmt(fmt(x)) == fmt(x).
pub fn format_items(items: &[Item]) -> String {
    print::print_items(items)
}
