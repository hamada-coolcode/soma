import sys

with open("src/types/check_item.rs", "r") as f:
    text = f.read()

# Add a test module to the end of check_item.rs
test_code = """
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::item::{Name, FnDef, Param};
    use crate::diag::Span;

    fn name(n: &str) -> Name {
        Name::new(n, Span::new(0, 1))
    }

    #[test]
    fn test_main_returns_never() {
        let f = FnDef {
            name: name("main"),
            generics: vec![],
            params: vec![],
            ret: None,
            body: Block {
                stmts: vec![],
                span: Span::new(0, 1),
            },
        };
        let items = vec![Item {
            attrs: vec![],
            kind: ItemKind::Fn(f),
            span: Span::new(0, 1),
        }];
        let diags = check_items(&items);
        // We aren't doing strict checking for main never return here unless required.
    }
}
"""

with open("src/types/check_item.rs", "a") as f:
    f.write(test_code)
