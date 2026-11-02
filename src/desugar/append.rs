use crate::ast::item::Item;

/// Lower `L + x` / `L + M` into `L.add(x)` / `L.extend(M)` calls.
/// TODO(task 11).
pub fn rewrite(items: Vec<Item>) -> Vec<Item> {
    items
}
