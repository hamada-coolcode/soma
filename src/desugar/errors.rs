//! We desugar error propagation (`e!`) into explicit `If` structures.
//!
//! # Encoding
//! - To avoid double-evaluating the expression and breaking short-circuiting,
//!   we bind the result to a deterministic temporary variable `__err_N`.
//! - Plain propagate `e!` is replaced by an inline block expression:
//!   `if true => { __err_N = e; if is_error(__err_N) { return __err_N } __err_N }`
//!   This properly short-circuits and replaces the expression with the success value.
//! - Handler propagate `e ! handler` is formed when `e!` is an expression statement
//!   where `e` is a `Call`, and the next sibling statement is the handler.
//!   These two statements are rewritten into:
//!   `__err_N = e; if is_error(__err_N) { handler } else { __err_N }`

use crate::ast::expr::{Arg, Arm, Block, Expr, ExprKind, Stmt, StmtKind};
use crate::ast::item::{Item, ItemKind, Name};
use crate::diag::Span;

/// Lower `e!` and `e ! handler` into explicit error matching.
pub fn rewrite(items: Vec<Item>) -> Vec<Item> {
    let mut counter = 0;
    items
        .into_iter()
        .map(|item| rewrite_item(item, &mut counter))
        .collect()
}

fn name_with_span(text: &str, span: Span) -> Name {
    Name {
        text: text.to_string(),
        span,
    }
}

fn rewrite_item(item: Item, counter: &mut u32) -> Item {
    let kind = match item.kind {
        ItemKind::Fn(mut f) => {
            f.body = rewrite_block(f.body, counter);
            ItemKind::Fn(f)
        }
        ItemKind::Object(mut o) => {
            o.methods = o
                .methods
                .into_iter()
                .map(|mut m| {
                    m.body = rewrite_block(m.body, counter);
                    m
                })
                .collect();
            ItemKind::Object(o)
        }
        ItemKind::Test(mut t) => {
            t.body = rewrite_block(t.body, counter);
            ItemKind::Test(t)
        }
        ItemKind::Import(mut i) => {
            i.children = i
                .children
                .into_iter()
                .map(|c| rewrite_item(c, counter))
                .collect();
            ItemKind::Import(i)
        }
        ItemKind::Const(mut c) => {
            c.value = rewrite_expr(&c.value, counter);
            ItemKind::Const(c)
        }
        other => other,
    };
    Item {
        attrs: item.attrs,
        kind,
        span: item.span,
    }
}

fn rewrite_block(block: Block, counter: &mut u32) -> Block {
    let mut stmts = Vec::new();
    let mut i = 0;
    while i < block.stmts.len() {
        let stmt = &block.stmts[i];

        if let StmtKind::Expr(expr) = &stmt.kind
            && let ExprKind::Propagate(inner) = &expr.kind
                && let ExprKind::Call { .. } = &inner.kind
                    && i + 1 < block.stmts.len() {
                        let handler_stmt = block.stmts[i + 1].clone();
                        let rewritten_inner = rewrite_expr(inner, counter);
                        let rewritten_handler = rewrite_stmt(&handler_stmt, counter);

                        let tmp_name = format!("__err_{}", counter);
                        *counter += 1;
                        let tmp = Expr {
                            kind: ExprKind::Name(name_with_span(&tmp_name, expr.span)),
                            span: expr.span,
                        };

                        let assign = Stmt {
                            kind: StmtKind::Assign {
                                target: tmp.clone(),
                                value: rewritten_inner,
                            },
                            span: expr.span,
                        };

                        let cond = make_is_error_call(tmp.clone(), expr.span);

                        let then_block = Block {
                            stmts: vec![rewritten_handler],
                            span: handler_stmt.span,
                        };

                        let else_block = Block {
                            stmts: vec![Stmt {
                                kind: StmtKind::Expr(tmp.clone()),
                                span: inner.span,
                            }],
                            span: inner.span,
                        };

                        let if_expr = Expr {
                            kind: ExprKind::If {
                                cond: Box::new(cond),
                                then: then_block,
                                elifs: vec![],
                                else_: Some(else_block),
                            },
                            span: expr.span,
                        };

                        stmts.push(assign);
                        stmts.push(Stmt {
                            kind: StmtKind::Expr(if_expr),
                            span: expr.span,
                        });

                        i += 2;
                        continue;
                    }

        stmts.push(rewrite_stmt(stmt, counter));
        i += 1;
    }

    Block {
        stmts,
        span: block.span,
    }
}

fn rewrite_stmt(stmt: &Stmt, counter: &mut u32) -> Stmt {
    let kind = match &stmt.kind {
        StmtKind::Expr(e) => StmtKind::Expr(rewrite_expr(e, counter)),
        StmtKind::Assign { target, value } => StmtKind::Assign {
            target: rewrite_expr(target, counter),
            value: rewrite_expr(value, counter),
        },
        StmtKind::While { cond, body } => StmtKind::While {
            cond: rewrite_expr(cond, counter),
            body: rewrite_block(body.clone(), counter),
        },
        StmtKind::For { var, iter, body } => StmtKind::For {
            var: var.clone(),
            iter: rewrite_expr(iter, counter),
            body: rewrite_block(body.clone(), counter),
        },
        StmtKind::Loop { body } => StmtKind::Loop {
            body: rewrite_block(body.clone(), counter),
        },
        StmtKind::Break => StmtKind::Break,
        StmtKind::Continue => StmtKind::Continue,
        StmtKind::Return(opt_e) => {
            StmtKind::Return(opt_e.as_ref().map(|e| rewrite_expr(e, counter)))
        }
        StmtKind::Match { scrutinee, arms } => StmtKind::Match {
            scrutinee: rewrite_expr(scrutinee, counter),
            arms: arms
                .iter()
                .map(|arm| Arm {
                    pattern: arm.pattern.clone(),
                    body: rewrite_stmt(&arm.body, counter),
                    span: arm.span,
                })
                .collect(),
        },
        StmtKind::Item(item) => StmtKind::Item(rewrite_item(item.clone(), counter)),
    };
    Stmt {
        kind,
        span: stmt.span,
    }
}

fn rewrite_expr(expr: &Expr, counter: &mut u32) -> Expr {
    let kind = match &expr.kind {
        ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Str(_)
        | ExprKind::FStr(_)
        | ExprKind::Bool(_)
        | ExprKind::Null
        | ExprKind::Name(_)
        | ExprKind::SelfExpr => {
            return expr.clone();
        }
        ExprKind::List(items) => {
            ExprKind::List(items.iter().map(|e| rewrite_expr(e, counter)).collect())
        }
        ExprKind::Call { callee, args } => ExprKind::Call {
            callee: Box::new(rewrite_expr(callee, counter)),
            args: args
                .iter()
                .map(|arg| Arg {
                    name: arg.name.clone(),
                    value: rewrite_expr(&arg.value, counter),
                })
                .collect(),
        },
        ExprKind::Index { base, index } => ExprKind::Index {
            base: Box::new(rewrite_expr(base, counter)),
            index: Box::new(rewrite_expr(index, counter)),
        },
        ExprKind::Field { base, name } => ExprKind::Field {
            base: Box::new(rewrite_expr(base, counter)),
            name: name.clone(),
        },
        ExprKind::Unary { op, operand } => ExprKind::Unary {
            op: *op,
            operand: Box::new(rewrite_expr(operand, counter)),
        },
        ExprKind::Binary { op, lhs, rhs } => ExprKind::Binary {
            op: *op,
            lhs: Box::new(rewrite_expr(lhs, counter)),
            rhs: Box::new(rewrite_expr(rhs, counter)),
        },
        ExprKind::Range { from, to } => ExprKind::Range {
            from: from.as_ref().map(|e| Box::new(rewrite_expr(e, counter))),
            to: to.as_ref().map(|e| Box::new(rewrite_expr(e, counter))),
        },
        ExprKind::Lambda { params, body } => ExprKind::Lambda {
            params: params.clone(),
            body: Box::new(rewrite_expr(body, counter)),
        },
        ExprKind::Comprehension {
            expr: c_expr,
            var,
            iter,
        } => ExprKind::Comprehension {
            expr: Box::new(rewrite_expr(c_expr, counter)),
            var: var.clone(),
            iter: Box::new(rewrite_expr(iter, counter)),
        },
        ExprKind::If {
            cond,
            then,
            elifs,
            else_,
        } => ExprKind::If {
            cond: Box::new(rewrite_expr(cond, counter)),
            then: rewrite_block(then.clone(), counter),
            elifs: elifs
                .iter()
                .map(|(c, b)| (rewrite_expr(c, counter), rewrite_block(b.clone(), counter)))
                .collect(),
            else_: else_.as_ref().map(|b| rewrite_block(b.clone(), counter)),
        },
        ExprKind::Propagate(inner) => {
            let rewritten_inner = rewrite_expr(inner, counter);

            let tmp_name = format!("__err_{}", counter);
            *counter += 1;
            let tmp = Expr {
                kind: ExprKind::Name(name_with_span(&tmp_name, expr.span)),
                span: expr.span,
            };

            let assign = Stmt {
                kind: StmtKind::Assign {
                    target: tmp.clone(),
                    value: rewritten_inner,
                },
                span: expr.span,
            };

            let check_if = Expr {
                kind: ExprKind::If {
                    cond: Box::new(make_is_error_call(tmp.clone(), expr.span)),
                    then: Block {
                        stmts: vec![Stmt {
                            kind: StmtKind::Return(Some(tmp.clone())),
                            span: expr.span,
                        }],
                        span: expr.span,
                    },
                    elifs: vec![],
                    else_: None,
                },
                span: expr.span,
            };
            let check_stmt = Stmt {
                kind: StmtKind::Expr(check_if),
                span: expr.span,
            };

            let success_val = Stmt {
                kind: StmtKind::Expr(tmp.clone()),
                span: expr.span,
            };

            let block = Block {
                stmts: vec![assign, check_stmt, success_val],
                span: expr.span,
            };

            return Expr {
                kind: ExprKind::If {
                    cond: Box::new(Expr {
                        kind: ExprKind::Bool(true),
                        span: expr.span,
                    }),
                    then: block,
                    elifs: vec![],
                    else_: None,
                },
                span: expr.span,
            };
        }
    };
    Expr {
        kind,
        span: expr.span,
    }
}

fn make_is_error_call(arg: Expr, span: Span) -> Expr {
    Expr {
        kind: ExprKind::Call {
            callee: Box::new(Expr {
                kind: ExprKind::Name(name_with_span("is_error", span)),
                span,
            }),
            args: vec![Arg {
                name: None,
                value: arg,
            }],
        },
        span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::item::{Item, ItemKind, FnDef, Name};

    use crate::diag::Span;

    fn span() -> Span {
        Span::default()
    }

    fn name(text: &str) -> Name {
        Name { text: text.to_string(), span: span() }
    }

    fn expr_name(text: &str) -> Expr {
        Expr { kind: ExprKind::Name(name(text)), span: span() }
    }

    fn expr_call(callee: Expr, args: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Call {
                callee: Box::new(callee),
                args: args.into_iter().map(|value| Arg { name: None, value }).collect(),
            },
            span: span(),
        }
    }

    fn stmt_expr(e: Expr) -> Stmt {
        Stmt { kind: StmtKind::Expr(e), span: span() }
    }

    fn item_fn(name_str: &str, body: Block) -> Item {
        Item {
            attrs: vec![],
            kind: ItemKind::Fn(FnDef {
                name: name(name_str),
                generics: vec![],
                params: vec![],
                ret: None,
                body,
            }),
            span: span(),
        }
    }

    #[test]
    fn test_plain_propagate() {
        // fn test() { x! }
        let call_x = expr_name("x");
        let prop = Expr { kind: ExprKind::Propagate(Box::new(call_x)), span: span() };
        let body = Block { stmts: vec![stmt_expr(prop)], span: span() };
        let items = vec![item_fn("test", body)];

        let rewritten = rewrite(items);
        if let ItemKind::Fn(f) = &rewritten[0].kind {
            assert_eq!(f.body.stmts.len(), 1);
            let s1 = &f.body.stmts[0];

            if let StmtKind::Expr(Expr { kind: ExprKind::If { cond, then, elifs, else_ }, .. }) = &s1.kind {
                assert!(elifs.is_empty());
                assert!(else_.is_none());

                // cond should be true
                if let ExprKind::Bool(b) = &cond.kind {
                    assert!(*b);
                } else { panic!("not true"); }

                assert_eq!(then.stmts.len(), 3);
            } else { panic!("not if"); }
        } else { panic!("not fn"); }
    }

    #[test]
    fn test_handler_sibling() {
        // fn test() { f()! \n handler() }
        let f_call = expr_call(expr_name("f"), vec![]);
        let prop = Expr { kind: ExprKind::Propagate(Box::new(f_call)), span: span() };
        let handler = expr_call(expr_name("handler"), vec![]);

        let body = Block {
            stmts: vec![stmt_expr(prop), stmt_expr(handler)],
            span: span(),
        };
        let items = vec![item_fn("test", body)];
        let rewritten = rewrite(items);

        if let ItemKind::Fn(f) = &rewritten[0].kind {
            assert_eq!(f.body.stmts.len(), 2); // Assign and If!

            // first is Assign
            if let StmtKind::Assign { .. } = &f.body.stmts[0].kind {
                // ok
            } else { panic!("not assign"); }

            // second is If
            if let StmtKind::Expr(Expr { kind: ExprKind::If { then, else_: Some(el), .. }, .. }) = &f.body.stmts[1].kind {
                assert_eq!(then.stmts.len(), 1);
                assert_eq!(el.stmts.len(), 1);
            } else { panic!("not if-else"); }
        }
    }
}
