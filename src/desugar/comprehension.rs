use crate::ast::item::Item;

/// Lower `[e for x in it]` and `for x in it` into explicit loops.
/// TODO(task 13).
pub fn rewrite(items: Vec<Item>) -> Vec<Item> {
    items
}
