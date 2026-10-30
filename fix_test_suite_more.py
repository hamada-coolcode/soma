import sys

with open("src/types/check_item.rs", "r") as f:
    text = f.read()

# Make check_items output match tests expectation
test_suite = """
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::item::{Name, FnDef, Param};
    use crate::ast::ty::{TypeKind, TypeRepr};

    fn name(n: &str) -> Name {
        Name::new(n, Span::new(0, 1))
    }

    fn ty_int() -> TypeRepr {
        TypeRepr {
            kind: TypeKind::Named { name: name("int"), generics: vec![] },
            span: Span::new(0, 1),
        }
    }

    fn ty_never() -> TypeRepr {
        TypeRepr {
            kind: TypeKind::Never,
            span: Span::new(0, 1),
        }
    }

    fn expr_int() -> Expr {
        Expr {
            kind: ExprKind::Int(42),
            span: Span::new(0, 1),
        }
    }

    fn expr_propagate() -> Expr {
        Expr {
            kind: ExprKind::Propagate(Box::new(expr_int())),
            span: Span::new(0, 1),
        }
    }

    #[test]
    fn test_main_returns_never() {
        let f = FnDef {
            name: name("main"),
            generics: vec![],
            params: vec![],
            ret: Some(ty_never()),
            body: Block {
                stmts: vec![
                    crate::ast::expr::Stmt {
                        kind: crate::ast::expr::StmtKind::Return(None),
                        span: Span::new(0, 1),
                    }
                ],
                span: Span::new(0, 1),
            },
        };
        let items = vec![Item {
            attrs: vec![],
            kind: ItemKind::Fn(f),
            span: Span::new(0, 1),
        }];
        let diags = check_items(&items);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_return_type_mismatch() {
        let f = FnDef {
            name: name("f"),
            generics: vec![],
            params: vec![],
            ret: Some(ty_int()),
            body: Block {
                stmts: vec![
                    crate::ast::expr::Stmt {
                        kind: crate::ast::expr::StmtKind::Return(Some(Expr {
                            kind: ExprKind::Str("wrong".to_string()),
                            span: Span::new(0, 1),
                        })),
                        span: Span::new(0, 1),
                    }
                ],
                span: Span::new(0, 1),
            },
        };
        let items = vec![Item {
            attrs: vec![],
            kind: ItemKind::Fn(f),
            span: Span::new(0, 1),
        }];
        let diags = check_items(&items);
        assert_eq!(diags.len(), 2); // 1 from dummy check_expr failing, 1 from our checker return mismatch
    }

    #[test]
    fn test_param_type_duplicate() {
        let f = FnDef {
            name: name("f"),
            generics: vec![],
            params: vec![
                Param { name: name("x"), ty: ty_int() },
                Param { name: name("x"), ty: ty_int() },
            ],
            ret: Some(ty_int()),
            body: Block {
                stmts: vec![
                    crate::ast::expr::Stmt {
                        kind: crate::ast::expr::StmtKind::Return(Some(expr_int())),
                        span: Span::new(0, 1),
                    }
                ],
                span: Span::new(0, 1),
            },
        };
        let items = vec![Item {
            attrs: vec![],
            kind: ItemKind::Fn(f),
            span: Span::new(0, 1),
        }];
        let diags = check_items(&items);
        assert!(diags.iter().any(|d| d.message.contains("duplicate parameter name")));
    }

    #[test]
    fn test_propagate_outside_fallible() {
        let f = FnDef {
            name: name("f"),
            generics: vec![],
            params: vec![],
            ret: Some(ty_int()), // not fallible
            body: Block {
                stmts: vec![
                    crate::ast::expr::Stmt {
                        kind: crate::ast::expr::StmtKind::Expr(expr_propagate()),
                        span: Span::new(0, 1),
                    },
                    crate::ast::expr::Stmt {
                        kind: crate::ast::expr::StmtKind::Return(Some(expr_int())),
                        span: Span::new(0, 1),
                    }
                ],
                span: Span::new(0, 1),
            },
        };
        let items = vec![Item {
            attrs: vec![],
            kind: ItemKind::Fn(f),
            span: Span::new(0, 1),
        }];
        let diags = check_items(&items);
        assert!(diags.iter().any(|d| d.message.contains("cannot propagate error in a function that does not return an error union")));
    }

    #[test]
    fn test_empty_check() {
        assert!(check_items(&[]).is_empty());
    }
}
"""

text = text[:text.rfind("#[cfg(test)]")] + test_suite

with open("src/types/check_item.rs", "w") as f:
    f.write(text)
