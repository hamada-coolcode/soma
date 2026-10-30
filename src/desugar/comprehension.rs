use crate::ast::item::*;
use crate::ast::expr::*;


/// Lower `[e for x in it]` and `for x in it` into explicit loops.
pub fn rewrite(items: Vec<Item>) -> Vec<Item> {
    items.into_iter().map(rewrite_item).collect()
}

fn rewrite_item(mut item: Item) -> Item {
    match &mut item.kind {
        ItemKind::Fn(f) => {
            f.body = rewrite_block(f.body.clone());
        }
        ItemKind::Object(o) => {
            for m in &mut o.methods {
                m.body = rewrite_block(m.body.clone());
            }
        }
        ItemKind::Trait(_) => {}
        ItemKind::Error(_) => {}
        ItemKind::Enum(_) => {}
        ItemKind::Import(i) => {
            i.children = i.children.clone().into_iter().map(rewrite_item).collect();
        }
        ItemKind::Const(c) => {
            c.value = rewrite_expr(c.value.clone());
        }
        ItemKind::Test(t) => {
            t.body = rewrite_block(t.body.clone());
        }
    }
    item
}

fn rewrite_block(mut block: Block) -> Block {
    let mut new_stmts = Vec::new();
    for stmt in block.stmts {
        new_stmts.extend(rewrite_stmt(stmt));
    }
    block.stmts = new_stmts;
    block
}

fn rewrite_stmt(stmt: Stmt) -> Vec<Stmt> {
    let span = stmt.span;
    match stmt.kind {
        StmtKind::Expr(e) => vec![Stmt { kind: StmtKind::Expr(rewrite_expr(e)), span }],
        StmtKind::Assign { target, value } => vec![Stmt {
            kind: StmtKind::Assign {
                target: rewrite_expr(target),
                value: rewrite_expr(value),
            },
            span,
        }],
        StmtKind::While { cond, body } => vec![Stmt {
            kind: StmtKind::While {
                cond: rewrite_expr(cond),
                body: rewrite_block(body),
            },
            span,
        }],
        StmtKind::For { var, iter, body } => {
            // Document the cursor convention:
            // __it = iter
            // while __it.has_next() {
            //     var = __it.next()
            //     body
            // }
            let iter = rewrite_expr(iter);
            let mut rewritten_body = rewrite_block(body);

            let it_name = Name::new(format!("__it_{}_{}", span.start, span.end), span);
            let it_expr = Expr {
                kind: ExprKind::Name(it_name.clone()),
                span,
            };

            let assign_it = Stmt {
                kind: StmtKind::Assign {
                    target: it_expr.clone(),
                    value: iter,
                },
                span,
            };

            let has_next_call = Expr {
                kind: ExprKind::Call {
                    callee: Box::new(Expr {
                        kind: ExprKind::Field {
                            base: Box::new(it_expr.clone()),
                            name: Name::new("has_next", span),
                        },
                        span,
                    }),
                    args: vec![],
                },
                span,
            };

            let next_call = Expr {
                kind: ExprKind::Call {
                    callee: Box::new(Expr {
                        kind: ExprKind::Field {
                            base: Box::new(it_expr.clone()),
                            name: Name::new("next", span),
                        },
                        span,
                    }),
                    args: vec![],
                },
                span,
            };

            let var_expr = Expr {
                kind: ExprKind::Name(var.clone()),
                span,
            };

            let assign_var = Stmt {
                kind: StmtKind::Assign {
                    target: var_expr,
                    value: next_call,
                },
                span,
            };

            rewritten_body.stmts.insert(0, assign_var);

            let while_stmt = Stmt {
                kind: StmtKind::While {
                    cond: has_next_call,
                    body: rewritten_body,
                },
                span,
            };

            vec![assign_it, while_stmt]
        }
        StmtKind::Loop { body } => vec![Stmt {
            kind: StmtKind::Loop {
                body: rewrite_block(body),
            },
            span,
        }],
        StmtKind::Break => vec![stmt],
        StmtKind::Continue => vec![stmt],
        StmtKind::Return(e) => vec![Stmt {
            kind: StmtKind::Return(e.map(rewrite_expr)),
            span,
        }],
        StmtKind::Match { scrutinee, arms } => vec![Stmt {
            kind: StmtKind::Match {
                scrutinee: rewrite_expr(scrutinee),
                arms: arms.into_iter().map(rewrite_arm).collect(),
            },
            span,
        }],
        StmtKind::Item(item) => vec![Stmt {
            kind: StmtKind::Item(rewrite_item(item)),
            span,
        }],
    }
}

fn rewrite_arm(arm: Arm) -> Arm {
    let mut new_arm = arm.clone();
    let stmts = rewrite_stmt(arm.body);
    if stmts.len() == 1 {
        new_arm.body = stmts.into_iter().next().unwrap();
    } else {
        // Wrap into a loop that runs once and breaks. Wait, we can just use a block!
        // No, there is no StmtKind::Block.
        // We will wrap in a while loop that runs once.
        // But what if it breaks or continues inside?
        // Wait, loop { stmts; break } works perfectly for block scopes in blockless languages.
        // Except StmtKind::Loop takes a Block.
        let block = Block {
            stmts: {
                let mut s = stmts;
                s.push(Stmt { kind: StmtKind::Break, span: arm.span });
                s
            },
            span: arm.span,
        };
        new_arm.body = Stmt {
            kind: StmtKind::Loop { body: block },
            span: arm.span,
        };
    }
    new_arm
}

fn rewrite_expr(expr: Expr) -> Expr {
    let span = expr.span;
    match expr.kind {
        ExprKind::List(exprs) => Expr {
            kind: ExprKind::List(exprs.into_iter().map(rewrite_expr).collect()),
            span,
        },
        ExprKind::Call { callee, args } => Expr {
            kind: ExprKind::Call {
                callee: Box::new(rewrite_expr(*callee)),
                args: args.into_iter().map(|arg| Arg { name: arg.name, value: rewrite_expr(arg.value) }).collect(),
            },
            span,
        },
        ExprKind::Index { base, index } => Expr {
            kind: ExprKind::Index {
                base: Box::new(rewrite_expr(*base)),
                index: Box::new(rewrite_expr(*index)),
            },
            span,
        },
        ExprKind::Field { base, name } => Expr {
            kind: ExprKind::Field {
                base: Box::new(rewrite_expr(*base)),
                name,
            },
            span,
        },
        ExprKind::Unary { op, operand } => Expr {
            kind: ExprKind::Unary {
                op,
                operand: Box::new(rewrite_expr(*operand)),
            },
            span,
        },
        ExprKind::Binary { op, lhs, rhs } => Expr {
            kind: ExprKind::Binary {
                op,
                lhs: Box::new(rewrite_expr(*lhs)),
                rhs: Box::new(rewrite_expr(*rhs)),
            },
            span,
        },
        ExprKind::Range { from, to } => Expr {
            kind: ExprKind::Range {
                from: from.map(|e| Box::new(rewrite_expr(*e))),
                to: to.map(|e| Box::new(rewrite_expr(*e))),
            },
            span,
        },
        ExprKind::Lambda { params, body } => Expr {
            kind: ExprKind::Lambda {
                params,
                body: Box::new(rewrite_expr(*body)),
            },
            span,
        },
        ExprKind::Comprehension { expr, var, iter } => {
            // [expr for var in iter] -> Call { callee: Name("__comprehension"), args: [Lambda { params: [var], body: expr }, iter] }
            let rewritten_expr = rewrite_expr(*expr);
            let rewritten_iter = rewrite_expr(*iter);

            let lambda = Expr {
                kind: ExprKind::Lambda {
                    params: vec![var.clone()],
                    body: Box::new(rewritten_expr),
                },
                span,
            };

            let callee = Expr {
                kind: ExprKind::Name(Name::new("__comprehension", span)),
                span,
            };

            Expr {
                kind: ExprKind::Call {
                    callee: Box::new(callee),
                    args: vec![
                        Arg { name: None, value: lambda },
                        Arg { name: None, value: rewritten_iter },
                    ],
                },
                span,
            }
        },
        ExprKind::Propagate(e) => Expr {
            kind: ExprKind::Propagate(Box::new(rewrite_expr(*e))),
            span,
        },
        ExprKind::If { cond, then, elifs, else_ } => Expr {
            kind: ExprKind::If {
                cond: Box::new(rewrite_expr(*cond)),
                then: rewrite_block(then),
                elifs: elifs.into_iter().map(|(c, b)| (rewrite_expr(c), rewrite_block(b))).collect(),
                else_: else_.map(rewrite_block),
            },
            span,
        },
        _ => expr,
    }
}

#[cfg(test)]
mod tests {
    use crate::diag::Span;
    use super::*;


    fn span(start: usize, end: usize) -> Span {
        Span { start, end }
    }

    fn ident(text: &str, span: Span) -> Name {
        Name::new(text, span)
    }

    fn expr_name(text: &str, span: Span) -> Expr {
        Expr {
            kind: ExprKind::Name(ident(text, span)),
            span,
        }
    }

    fn expr_int(val: i64, span: Span) -> Expr {
        Expr {
            kind: ExprKind::Int(val),
            span,
        }
    }

    #[test]
    fn test_comprehension_rewrite() {
        // [x * 2 for x in iter]
        let s = span(0, 20);
        let x_name = ident("x", s);

        let mul_expr = Expr {
            kind: ExprKind::Binary {
                op: BinOp::Mul,
                lhs: Box::new(expr_name("x", s)),
                rhs: Box::new(expr_int(2, s)),
            },
            span: s,
        };

        let comp = Expr {
            kind: ExprKind::Comprehension {
                expr: Box::new(mul_expr.clone()),
                var: x_name.clone(),
                iter: Box::new(expr_name("iter", s)),
            },
            span: s,
        };

        let rewritten = rewrite_expr(comp);

        match rewritten.kind {
            ExprKind::Call { callee, args } => {
                match callee.kind {
                    ExprKind::Name(n) => assert_eq!(n.text, "__comprehension"),
                    _ => panic!("Expected __comprehension callee"),
                }
                assert_eq!(args.len(), 2);

                match &args[0].value.kind {
                    ExprKind::Lambda { params, body } => {
                        assert_eq!(params.len(), 1);
                        assert_eq!(params[0].text, "x");
                        assert_eq!(**body, mul_expr);
                    }
                    _ => panic!("Expected Lambda"),
                }

                match &args[1].value.kind {
                    ExprKind::Name(n) => assert_eq!(n.text, "iter"),
                    _ => panic!("Expected Name"),
                }
            }
            _ => panic!("Expected Call"),
        }
        assert_eq!(rewritten.span, s);
    }

    #[test]
    fn test_for_rewrite() {
        // for x in iter { print(x) }
        let s = span(0, 20);
        let x_name = ident("x", s);

        let print_call = Expr {
            kind: ExprKind::Call {
                callee: Box::new(expr_name("print", s)),
                args: vec![Arg { name: None, value: expr_name("x", s) }],
            },
            span: s,
        };

        let for_stmt = Stmt {
            kind: StmtKind::For {
                var: x_name.clone(),
                iter: expr_name("iter", s),
                body: Block {
                    stmts: vec![Stmt { kind: StmtKind::Expr(print_call.clone()), span: s }],
                    span: s,
                },
            },
            span: s,
        };

        let rewritten = rewrite_stmt(for_stmt);

        assert_eq!(rewritten.len(), 2);

        // __it = iter
        match &rewritten[0].kind {
            StmtKind::Assign { target, value } => {
                match &target.kind {
                    ExprKind::Name(n) => assert!(n.text.starts_with("__it")),
                    _ => panic!("Expected Name"),
                }
                match &value.kind {
                    ExprKind::Name(n) => assert_eq!(n.text, "iter"),
                    _ => panic!("Expected Name"),
                }
            }
            _ => panic!("Expected Assign"),
        }

        // while __it.has_next() { x = __it.next(); print(x) }
        match &rewritten[1].kind {
            StmtKind::While { cond, body } => {
                match &cond.kind {
                    ExprKind::Call { callee, args } => {
                        assert_eq!(args.len(), 0);
                        match &callee.kind {
                            ExprKind::Field { base, name } => {
                                match &base.kind {
                                    ExprKind::Name(n) => assert!(n.text.starts_with("__it")),
                                    _ => panic!("Expected Name"),
                                }
                                assert_eq!(name.text, "has_next");
                            }
                            _ => panic!("Expected Field"),
                        }
                    }
                    _ => panic!("Expected Call"),
                }

                assert_eq!(body.stmts.len(), 2);

                // x = __it.next()
                match &body.stmts[0].kind {
                    StmtKind::Assign { target, value } => {
                        match &target.kind {
                            ExprKind::Name(n) => assert_eq!(n.text, "x"),
                            _ => panic!("Expected Name"),
                        }
                        match &value.kind {
                            ExprKind::Call { callee, args } => {
                                assert_eq!(args.len(), 0);
                                match &callee.kind {
                                    ExprKind::Field { base, name } => {
                                        match &base.kind {
                                            ExprKind::Name(n) => assert!(n.text.starts_with("__it")),
                                            _ => panic!("Expected Name"),
                                        }
                                        assert_eq!(name.text, "next");
                                    }
                                    _ => panic!("Expected Field"),
                                }
                            }
                            _ => panic!("Expected Call"),
                        }
                    }
                    _ => panic!("Expected Assign"),
                }

                // print(x)
                assert_eq!(body.stmts[1].kind, StmtKind::Expr(print_call));
            }
            _ => panic!("Expected While"),
        }
    }

}
#[cfg(test)]
mod additional_tests {
    use crate::diag::Span;
    use super::*;


    fn span(start: usize, end: usize) -> Span {
        Span { start, end }
    }

    fn ident(text: &str, span: Span) -> Name {
        Name::new(text, span)
    }

    fn expr_name(text: &str, span: Span) -> Expr {
        Expr {
            kind: ExprKind::Name(ident(text, span)),
            span,
        }
    }

    #[test]
    fn test_nested_comprehension_in_fn() {
        let s = span(0, 50);

        let comp = Expr {
            kind: ExprKind::Comprehension {
                expr: Box::new(expr_name("x", s)),
                var: ident("x", s),
                iter: Box::new(expr_name("iter", s)),
            },
            span: s,
        };

        let stmt = Stmt {
            kind: StmtKind::Expr(comp),
            span: s,
        };

        let fn_def = FnDef {
            name: ident("test_func", s),
            generics: vec![],
            params: vec![],
            ret: None,
            body: Block {
                stmts: vec![stmt],
                span: s,
            },
        };

        let item = Item {
            attrs: vec![],
            kind: ItemKind::Fn(fn_def),
            span: s,
        };

        let rewritten_items = rewrite(vec![item]);
        assert_eq!(rewritten_items.len(), 1);

        match &rewritten_items[0].kind {
            ItemKind::Fn(f) => {
                assert_eq!(f.body.stmts.len(), 1);
                match &f.body.stmts[0].kind {
                    StmtKind::Expr(e) => {
                        match e.kind {
                            ExprKind::Call { .. } => {}
                            _ => panic!("Expected Call inside function body"),
                        }
                    }
                    _ => panic!("Expected Expr stmt"),
                }
            }
            _ => panic!("Expected Fn"),
        }
    }
}
