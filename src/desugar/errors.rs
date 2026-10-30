use crate::ast::item::Item;

/// Lower `e!` and `e ! handler` into explicit error matching.
/// TODO(task 12).
pub fn rewrite(items: Vec<Item>) -> Vec<Item> {
    items
}
