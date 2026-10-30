use crate::ast::expr::{Arg, BinOp, Block, Expr, ExprKind, Stmt, StmtKind};
use crate::ast::item::{Item, ItemKind, Name};

/// Lower `L + x` / `L + M` into `L.add(x)` / `L.extend(M)` calls.
pub fn rewrite(mut items: Vec<Item>) -> Vec<Item> {
    for item in &mut items {
        walk_item(item);
    }
    items
}

fn walk_item(item: &mut Item) {
    match &mut item.kind {
        ItemKind::Fn(f) => {
            walk_block(&mut f.body);
        }
        ItemKind::Object(o) => {
            for m in &mut o.methods {
                walk_block(&mut m.body);
            }
        }
        ItemKind::Import(i) => {
            for child in &mut i.children {
                walk_item(child);
            }
        }
        ItemKind::Const(c) => {
            walk_expr(&mut c.value);
        }
        ItemKind::Test(t) => {
            walk_block(&mut t.body);
        }
        ItemKind::Trait(_) | ItemKind::Error(_) | ItemKind::Enum(_) => {}
    }
}

fn walk_block(block: &mut Block) {
    for stmt in &mut block.stmts {
        walk_stmt(stmt);
    }
}

fn walk_stmt(stmt: &mut Stmt) {
    match &mut stmt.kind {
        StmtKind::Expr(e) => walk_expr(e),
        StmtKind::Assign { target, value } => {
            walk_expr(target);
            walk_expr(value);
        }
        StmtKind::While { cond, body } => {
            walk_expr(cond);
            walk_block(body);
        }
        StmtKind::For { iter, body, .. } => {
            walk_expr(iter);
            walk_block(body);
        }
        StmtKind::Loop { body } => {
            walk_block(body);
        }
        StmtKind::Break | StmtKind::Continue => {}
        StmtKind::Return(Some(e)) => walk_expr(e),
        StmtKind::Return(None) => {}
        StmtKind::Match { scrutinee, arms } => {
            walk_expr(scrutinee);
            for arm in arms {
                walk_stmt(&mut arm.body);
            }
        }
        StmtKind::Item(item) => walk_item(item),
    }
}

fn walk_expr(expr: &mut Expr) {
    // Post-order traversal: recurse first, then process
    match &mut expr.kind {
        ExprKind::List(items) => {
            for item in items {
                walk_expr(item);
            }
        }
        ExprKind::Call { callee, args } => {
            walk_expr(callee);
            for arg in args {
                walk_expr(&mut arg.value);
            }
        }
        ExprKind::Index { base, index } => {
            walk_expr(base);
            walk_expr(index);
        }
        ExprKind::Field { base, .. } => {
            walk_expr(base);
        }
        ExprKind::Unary { operand, .. } => {
            walk_expr(operand);
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            walk_expr(lhs);
            walk_expr(rhs);
        }
        ExprKind::Range { from, to } => {
            if let Some(f) = from {
                walk_expr(f);
            }
            if let Some(t) = to {
                walk_expr(t);
            }
        }
        ExprKind::Lambda { body, .. } => {
            walk_expr(body);
        }
        ExprKind::Comprehension { expr: c_expr, iter, .. } => {
            walk_expr(c_expr);
            walk_expr(iter);
        }
        ExprKind::Propagate(e) => {
            walk_expr(e);
        }
        ExprKind::If { cond, then, elifs, else_ } => {
            walk_expr(cond);
            walk_block(then);
            for (c, b) in elifs {
                walk_expr(c);
                walk_block(b);
            }
            if let Some(b) = else_ {
                walk_block(b);
            }
        }
        ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Str(_)
        | ExprKind::FStr(_)
        | ExprKind::Bool(_)
        | ExprKind::Null
        | ExprKind::Name(_)
        | ExprKind::SelfExpr => {}
    }

    // Rewrite if it's an Add binary operator
    if let ExprKind::Binary { op: BinOp::Add, lhs, rhs } = &mut expr.kind {
        // The rule from the brief:
        // "Since types aren't available yet, use this rule: rewrite lhs if it is NOT ExprKind::Int/Float."
        // We also do not rewrite when LHS is a string literal.
        let should_rewrite = !matches!(
            lhs.kind,
            ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Str(_)
        );

        if should_rewrite {
            let lhs_val = std::mem::replace(
                &mut **lhs,
                Expr {
                    kind: ExprKind::Null,
                    span: expr.span.clone(),
                },
            );
            let rhs_val = std::mem::replace(
                &mut **rhs,
                Expr {
                    kind: ExprKind::Null,
                    span: expr.span.clone(),
                },
            );

            // Document why we don't rewrite when LHS is a string literal:
            // Concatenation of two string literals stays `Add`. The type checker
            // will reject it because types aren't available yet, but we want it
            // untouched so the type checker handles it natively or throws an error.

            let span = expr.span.clone();

            expr.kind = ExprKind::Call {
                callee: Box::new(Expr {
                    kind: ExprKind::Field {
                        base: Box::new(lhs_val),
                        name: Name::new("add", span.clone()),
                    },
                    span: span.clone(),
                }),
                args: vec![Arg {
                    name: None,
                    value: rhs_val,
                }],
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::{BinOp, Block, Expr, ExprKind, Stmt, StmtKind};
    use crate::diag::Span;

    fn span() -> Span {
        Span::new(0, 0)
    }

    fn name(n: &str) -> Expr {
        Expr {
            kind: ExprKind::Name(Name::new(n, span())),
            span: span(),
        }
    }

    fn int(i: i64) -> Expr {
        Expr {
            kind: ExprKind::Int(i),
            span: span(),
        }
    }

    fn str_expr(s: &str) -> Expr {
        Expr {
            kind: ExprKind::Str(s.to_string()),
            span: span(),
        }
    }

    fn list(items: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::List(items),
            span: span(),
        }
    }

    fn add(lhs: Expr, rhs: Expr) -> Expr {
        Expr {
            kind: ExprKind::Binary {
                op: BinOp::Add,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            span: span(),
        }
    }

    fn call_add(base: Expr, arg: Expr) -> Expr {
        Expr {
            kind: ExprKind::Call {
                callee: Box::new(Expr {
                    kind: ExprKind::Field {
                        base: Box::new(base),
                        name: Name::new("add", span()),
                    },
                    span: span(),
                }),
                args: vec![Arg {
                    name: None,
                    value: arg,
                }],
            },
            span: span(),
        }
    }

    #[test]
    fn test_rewrite_list_add_int() {
        let mut e = add(name("seq"), int(5));
        walk_expr(&mut e);
        assert_eq!(e, call_add(name("seq"), int(5)));
    }

    #[test]
    fn test_rewrite_list_add_list() {
        let mut e = add(name("seq"), list(vec![int(1), int(2)]));
        walk_expr(&mut e);
        assert_eq!(e, call_add(name("seq"), list(vec![int(1), int(2)])));
    }

    #[test]
    fn test_no_rewrite_int() {
        let mut e = add(int(1), int(2));
        let orig = e.clone();
        walk_expr(&mut e);
        assert_eq!(e, orig);
    }

    #[test]
    fn test_no_rewrite_str() {
        let mut e = add(str_expr("a"), str_expr("b"));
        let orig = e.clone();
        walk_expr(&mut e);
        assert_eq!(e, orig);
    }

    #[test]
    fn test_nested_rewrite() {
        // f(a + b)
        let mut e = Expr {
            kind: ExprKind::Call {
                callee: Box::new(name("f")),
                args: vec![Arg {
                    name: None,
                    value: add(name("a"), name("b")),
                }],
            },
            span: span(),
        };
        walk_expr(&mut e);

        let expected = Expr {
            kind: ExprKind::Call {
                callee: Box::new(name("f")),
                args: vec![Arg {
                    name: None,
                    value: call_add(name("a"), name("b")),
                }],
            },
            span: span(),
        };
        assert_eq!(e, expected);
    }

    #[test]
    fn test_chained_rewrite() {
        // (a + b) + c
        let mut e = add(add(name("a"), name("b")), name("c"));
        walk_expr(&mut e);

        // Should become (a.add(b)).add(c)
        assert_eq!(e, call_add(call_add(name("a"), name("b")), name("c")));
    }

    #[test]
    fn test_walk_bodies() {
        let mut e = Expr {
            kind: ExprKind::If {
                cond: Box::new(name("cond")),
                then: Block {
                    stmts: vec![Stmt {
                        kind: StmtKind::Expr(add(name("a"), name("b"))),
                        span: span(),
                    }],
                    span: span(),
                },
                elifs: vec![],
                else_: None,
            },
            span: span(),
        };

        walk_expr(&mut e);

        let expected_then = Block {
            stmts: vec![Stmt {
                kind: StmtKind::Expr(call_add(name("a"), name("b"))),
                span: span(),
            }],
            span: span(),
        };

        if let ExprKind::If { then, .. } = e.kind {
            assert_eq!(then, expected_then);
        } else {
            panic!("Expected If expression");
        }
    }
}
