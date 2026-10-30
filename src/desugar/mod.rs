pub mod append;
pub mod comprehension;
pub mod errors;
pub mod psoma;

use crate::ast::item::Item;

/// Run every lowering pass in order. `.psoma` is the result of this pipeline.
///
/// `append` is deliberately not run here: deciding whether `+` means integer
/// addition or list append needs the type checker. Once types are available the
/// checker drives `append::rewrite`; until then `+` is preserved verbatim (the
/// interpreter implements list `+` directly).
pub fn lower(mut items: Vec<Item>) -> Vec<Item> {
    items = comprehension::rewrite(items);
    items = errors::rewrite(items);
    items
}
