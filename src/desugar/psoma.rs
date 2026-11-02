use crate::ast::item::Item;

/// Print the lowered AST as `.psoma`: explicit braces, semicolons, commas.
/// Must be stable and round-trippable for golden-file tests.
/// TODO(task 14).
pub fn emit(_items: &[Item]) -> String {
    String::new()
}
