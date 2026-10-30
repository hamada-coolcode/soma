pub mod append;
pub mod comprehension;
pub mod errors;
pub mod psoma;

use crate::ast::item::Item;

/// Run every lowering pass in order. `.psoma` is the result of this pipeline.
pub fn lower(mut items: Vec<Item>) -> Vec<Item> {
    items = append::rewrite(items);
    items = comprehension::rewrite(items);
    items = errors::rewrite(items);
    items
}
