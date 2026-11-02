use crate::ast::expr::{
    Arg, Arm, BinOp, Block, Expr, ExprKind, Pattern, Stmt, StmtKind, UnOp,
};
use crate::ast::item::Name;

use crate::lexer::{Op, TokKind};
use crate::parse::util::Parser;
use crate::parse::ParseError;

fn precedence(p: &Parser<'_>) -> Option<(u8, bool)> {
    let tok = p.peek()?;
    match &tok.kind {
        TokKind::Op(Op::FatArrow) => Some((1, true)),
        TokKind::Ident(s) if s == "or" => Some((2, false)),
        TokKind::Ident(s) if s == "and" => Some((3, false)),
        TokKind::Op(Op::DotDot) => Some((5, false)),
        TokKind::Op(Op::EqEq) | TokKind::Op(Op::BangEq) => Some((6, false)),
        TokKind::Op(Op::Lt) | TokKind::Op(Op::Gt) | TokKind::Op(Op::LtEq) | TokKind::Op(Op::GtEq) => {
            Some((7, false))
        }
        TokKind::Op(Op::Plus) | TokKind::Op(Op::Minus) => Some((8, false)),
        TokKind::Op(Op::Star) | TokKind::Op(Op::Slash) | TokKind::Op(Op::Percent) => {
            Some((9, false))
        }
        _ => None,
    }
}

pub fn expr(p: &mut Parser<'_>, min_prec: u8) -> Result<Expr, ParseError> {
    let mut lhs = parse_prefix_or_primary(p)?;

    while let Some((prec, right_assoc)) = precedence(p) {
        if prec < min_prec {
            break;
        }

        let op_tok = p.bump().unwrap().clone();
        let next_min_prec = if right_assoc { prec } else { prec + 1 };

        match op_tok.kind {
            TokKind::Op(Op::FatArrow) => {
                let mut params = Vec::new();
                match lhs.kind {
                    ExprKind::Name(name) => params.push(name),
                    ExprKind::List(items) => {
                        for item in items {
                            if let ExprKind::Name(name) = item.kind {
                                params.push(name);
                            } else {
                                return Err(ParseError::new("invalid lambda parameter", item.span));
                            }
                        }
                    }
                    ExprKind::Call { callee: _, args: _ } => {
                        // grammar says `(a b) => e` which might parse as a Call if we just use parens?
                        // Call args are separated by spaces: `call = "(" [ arg { arg } ] ")"`
                        // So `(a b)` is Call{ callee: a, args: [b] }? No wait, grammar says: `primary = ... | "(" expr ")"`
                        // If it's a call, the callee is `a`, and args are `[b]`.
                        // So `(a b) => e` starts with `(`.
                        // We should probably handle `(a b)` in primary parsing.
                        return Err(ParseError::new("invalid lambda parameters", op_tok.span));
                    }
                    _ => return Err(ParseError::new("invalid lambda parameters", lhs.span)),
                }
                let body = expr(p, next_min_prec)?;
                let span = lhs.span.join(body.span);
                lhs = Expr {
                    kind: ExprKind::Lambda { params, body: Box::new(body) },
                    span,
                };
            }
            TokKind::Op(Op::DotDot) => {
                let is_end = match p.peek_kind() {
                    None | Some(TokKind::Eof) | Some(TokKind::Newline) | Some(TokKind::Indent) | Some(TokKind::Dedent)
                    | Some(TokKind::Op(Op::RBracket)) | Some(TokKind::Op(Op::RParen)) => true,
                    _ => false,
                };
                let to = if is_end {
                    None
                } else {
                    Some(Box::new(expr(p, next_min_prec)?))
                };
                let span = if let Some(t) = &to {
                    lhs.span.join(t.span)
                } else {
                    lhs.span.join(op_tok.span)
                };
                lhs = Expr {
                    kind: ExprKind::Range {
                        from: Some(Box::new(lhs)),
                        to,
                    },
                    span,
                };
            }
            _ => {
                let op = match op_tok.kind {
                    TokKind::Ident(s) if s == "or" => BinOp::Or,
                    TokKind::Ident(s) if s == "and" => BinOp::And,
                    TokKind::Op(Op::EqEq) => BinOp::Eq,
                    TokKind::Op(Op::BangEq) => BinOp::Ne,
                    TokKind::Op(Op::Lt) => BinOp::Lt,
                    TokKind::Op(Op::Gt) => BinOp::Gt,
                    TokKind::Op(Op::LtEq) => BinOp::Le,
                    TokKind::Op(Op::GtEq) => BinOp::Ge,
                    TokKind::Op(Op::Plus) => BinOp::Add,
                    TokKind::Op(Op::Minus) => BinOp::Sub,
                    TokKind::Op(Op::Star) => BinOp::Mul,
                    TokKind::Op(Op::Slash) => BinOp::Div,
                    TokKind::Op(Op::Percent) => BinOp::Rem,
                    _ => unreachable!(),
                };
                let rhs = expr(p, next_min_prec)?;
                let span = lhs.span.join(rhs.span);
                lhs = Expr {
                    kind: ExprKind::Binary {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    span,
                };
            }
        }
    }

    Ok(lhs)
}

fn parse_prefix_or_primary(p: &mut Parser<'_>) -> Result<Expr, ParseError> {
    if let Some(tok) = p.peek() {
        match &tok.kind {
            TokKind::Ident(s) if s == "not" => {
                let span = tok.span;
                p.bump();
                let operand = expr(p, 4)?;
                return Ok(Expr {
                    span: span.join(operand.span),
                    kind: ExprKind::Unary {
                        op: UnOp::Not,
                        operand: Box::new(operand),
                    },
                });
            }
            TokKind::Op(Op::Minus) => {
                let span = tok.span;
                p.bump();
                let operand = expr(p, 10)?;
                return Ok(Expr {
                    span: span.join(operand.span),
                    kind: ExprKind::Unary {
                        op: UnOp::Neg,
                        operand: Box::new(operand),
                    },
                });
            }
            TokKind::Op(Op::DotDot) => {
                let span = tok.span;
                p.bump();
                let is_end = match p.peek_kind() {
                    None | Some(TokKind::Eof) | Some(TokKind::Newline) | Some(TokKind::Indent) | Some(TokKind::Dedent)
                    | Some(TokKind::Op(Op::RBracket)) | Some(TokKind::Op(Op::RParen)) => true,
                    _ => false,
                };
                let to = if is_end {
                    None
                } else {
                    Some(Box::new(expr(p, 5)?))
                };
                let full_span = if let Some(t) = &to {
                    span.join(t.span)
                } else {
                    span
                };
                return Ok(Expr {
                    span: full_span,
                    kind: ExprKind::Range { from: None, to },
                });
            }
            _ => {}
        }
    }

    let mut base = parse_primary(p)?;

    // postfix: call `(args)`, index `[e]`, field `.name`, propagate `!`
    loop {
        if let Some(tok) = p.peek() {
            match &tok.kind {
                TokKind::Op(Op::LParen) => {
                    p.bump();
                    let mut args = Vec::new();
                    while !p.check_op(Op::RParen) && !p.at_eof() {
                        let mut name = None;
                        // look ahead to see if named arg: `name = expr`
                        // since we can't lookahead easily, let's parse an expression.
                        // `p.toks.get(p.pos + 1)`
                        if let Some(TokKind::Ident(n)) = p.peek_kind() {
                            if let Some(next) = p.toks.get(p.pos + 1) {
                                if next.kind == TokKind::Op(Op::Assign) {
                                    let span = p.peek_span();
                                    name = Some(Name::new(n.clone(), span));
                                    p.bump(); // ident
                                    p.bump(); // =
                                }
                            }
                        }

                        let value = expr(p, 1)?;
                        args.push(Arg { name, value });
                    }
                    let end_span = p.expect_op(Op::RParen)?;
                    let span = base.span.join(end_span);
                    base = Expr {
                        kind: ExprKind::Call {
                            callee: Box::new(base),
                            args,
                        },
                        span,
                    };
                    continue;
                }
                TokKind::Op(Op::LBracket) => {
                    p.bump();
                    let index = expr(p, 1)?;
                    let end_span = p.expect_op(Op::RBracket)?;
                    let span = base.span.join(end_span);
                    base = Expr {
                        kind: ExprKind::Index {
                            base: Box::new(base),
                            index: Box::new(index),
                        },
                        span,
                    };
                    continue;
                }
                TokKind::Op(Op::Dot) => {
                    p.bump();
                    let name = p.expect_ident()?;
                    let span = base.span.join(name.span);
                    base = Expr {
                        kind: ExprKind::Field {
                            base: Box::new(base),
                            name,
                        },
                        span,
                    };
                    continue;
                }
                TokKind::Op(Op::Bang) => {
                    let end_span = p.bump().unwrap().span;
                    let span = base.span.join(end_span);
                    base = Expr {
                        kind: ExprKind::Propagate(Box::new(base)),
                        span,
                    };
                    continue;
                }
                _ => {}
            }
        }
        break;
    }

    Ok(base)
}

fn parse_primary(p: &mut Parser<'_>) -> Result<Expr, ParseError> {
    let tok = p.bump().cloned().ok_or_else(|| p.unexpected("expression"))?;
    match tok.kind {
        TokKind::Int(v) => Ok(Expr { span: tok.span, kind: ExprKind::Int(v) }),
        TokKind::Float(v) => Ok(Expr { span: tok.span, kind: ExprKind::Float(v) }),
        TokKind::Str(s) => Ok(Expr { span: tok.span, kind: ExprKind::Str(s) }),
        TokKind::FStr(parts) => Ok(Expr { span: tok.span, kind: ExprKind::FStr(parts) }),
        TokKind::Ident(s) => {
            if s == "true" {
                Ok(Expr { span: tok.span, kind: ExprKind::Bool(true) })
            } else if s == "false" {
                Ok(Expr { span: tok.span, kind: ExprKind::Bool(false) })
            } else if s == "null" {
                Ok(Expr { span: tok.span, kind: ExprKind::Null })
            } else if s == "self" {
                Ok(Expr { span: tok.span, kind: ExprKind::SelfExpr })
            } else if s == "if" {
                // `if` expr `=>` expr [ `else` expr ]
                let cond = expr(p, 1)?;
                p.expect_op(Op::FatArrow)?;
                let then_expr = expr(p, 1)?;

                let mut else_expr = None;
                if p.eat_kw("else") {
                    else_expr = Some(expr(p, 1)?);
                }

                // Construct a block from then_expr for ExprKind::If
                let then_block = Block {
                    stmts: vec![Stmt {
                        kind: StmtKind::Expr(then_expr.clone()),
                        span: then_expr.span,
                    }],
                    span: then_expr.span,
                };

                let else_block = else_expr.as_ref().map(|e| Block {
                    stmts: vec![Stmt {
                        kind: StmtKind::Expr(e.clone()),
                        span: e.span,
                    }],
                    span: e.span,
                });

                let span = if let Some(e) = &else_expr {
                    tok.span.join(e.span)
                } else {
                    tok.span.join(then_expr.span)
                };

                Ok(Expr {
                    span,
                    kind: ExprKind::If {
                        cond: Box::new(cond),
                        then: then_block,
                        elifs: vec![],
                        else_: else_block,
                    },
                })
            } else {
                Ok(Expr { span: tok.span, kind: ExprKind::Name(Name::new(s, tok.span)) })
            }
        }
        TokKind::Op(Op::LParen) => {
            // Need to distinguish `( expr )` and `( a b c )` (lambda params)
            // If it's a list of identifiers without spaces it might be lambda params.
            // To simplify, if we see identifiers separated by space, it's lambda.
            // But how do we distinguish `( a )` which could be an expr or lambda param?
            // If the next token after `)` is `=>`, it's a lambda.
            let mut idents_only = true;
            let mut idents = Vec::new();

            // Lookahead to check for lambda
            let mut i = p.pos; // after '('
            while let Some(t) = p.toks.get(i) {
                if t.kind == TokKind::Op(Op::RParen) {
                    if let Some(next) = p.toks.get(i + 1) {
                        if next.kind == TokKind::Op(Op::FatArrow) {
                            // It's definitely a lambda!
                            break;
                        }
                    }
                    // It's not a lambda if not followed by =>?
                    idents_only = false;
                    break;
                }
                match &t.kind {
                    TokKind::Ident(s) => idents.push(Name::new(s.clone(), t.span)),
                    _ => idents_only = false,
                }
                i += 1;
            }

            if idents_only {
                // It is a lambda param list
                let mut params = Vec::new();
                while !p.check_op(Op::RParen) && !p.at_eof() {
                    params.push(p.expect_ident()?);
                }
                let end_span = p.expect_op(Op::RParen)?;
                let span = tok.span.join(end_span);

                // Return a special List or something? But we need a way to return `(a b)`.
                // If we return a List, it could be misinterpreted.
                if p.check_op(Op::FatArrow) {
                    // It is a lambda! We handle it here directly?
                    // We can use `ExprKind::List` for `(a b)` just temporarily?
                    // Actually, if we use `ExprKind::List` for lambda params, we can extract them in `expr`.
                    // But `[a b c]` is a real List.
                    return Ok(Expr {
                        span,
                        kind: ExprKind::List(params.into_iter().map(|n| Expr {
                            span: n.span,
                            kind: ExprKind::Name(n),
                        }).collect()),
                    });
                }
            }

            // Otherwise, it's an `( expr )`
            let e = expr(p, 1)?;
            let end_span = p.expect_op(Op::RParen)?;
            Ok(Expr {
                span: tok.span.join(end_span),
                kind: e.kind,
            })
        }
        TokKind::Op(Op::LBracket) => {
            // list or comprehension
            let mut items = Vec::new();
            if p.check_op(Op::RBracket) {
                let end_span = p.bump().unwrap().span;
                return Ok(Expr {
                    span: tok.span.join(end_span),
                    kind: ExprKind::List(vec![]),
                });
            }

            let first = expr(p, 1)?;

            if p.eat_kw("for") {
                // comprehension: `[e for x in it]`
                let var = p.expect_ident()?;
                if !p.eat_kw("in") {
                    return Err(p.unexpected("in"));
                }
                let iter = expr(p, 1)?;
                let end_span = p.expect_op(Op::RBracket)?;
                return Ok(Expr {
                    span: tok.span.join(end_span),
                    kind: ExprKind::Comprehension {
                        expr: Box::new(first),
                        var,
                        iter: Box::new(iter),
                    },
                });
            } else {
                items.push(first);
                while !p.check_op(Op::RBracket) && !p.at_eof() {
                    items.push(expr(p, 1)?);
                }
                let end_span = p.expect_op(Op::RBracket)?;
                return Ok(Expr {
                    span: tok.span.join(end_span),
                    kind: ExprKind::List(items),
                });
            }
        }
        _ => Err(ParseError::new("expected expression", tok.span)),
    }
}

pub fn stmt(p: &mut Parser<'_>) -> Result<Stmt, ParseError> {
    if let Some(tok) = p.peek() {
        match &tok.kind {
            TokKind::Ident(s) => match s.as_str() {
                "while" => return parse_while(p),
                "for" => return parse_for(p),
                "loop" => return parse_loop(p),
                "break" => {
                    let span = tok.span;
                    p.bump();
                    return Ok(Stmt {
                        kind: StmtKind::Break,
                        span,
                    });
                }
                "continue" => {
                    let span = tok.span;
                    p.bump();
                    return Ok(Stmt {
                        kind: StmtKind::Continue,
                        span,
                    });
                }
                "return" => return parse_return(p),
                "if" => return parse_if(p),
                "match" => return parse_match(p),
                _ => {}
            },
            _ => {}
        }
    }

    // Default to expr or assign
    let expr_val = expr(p, 1)?;

    // Check if it's followed by '='
    if p.check_op(Op::Assign) {
        p.bump();
        let value = expr(p, 1)?;
        let span = expr_val.span.join(value.span);
        return Ok(Stmt {
            kind: StmtKind::Assign { target: expr_val, value },
            span,
        });
    }

    // Otherwise, expression statement
    let span = expr_val.span;
    Ok(Stmt {
        kind: StmtKind::Expr(expr_val),
        span,
    })
}

fn parse_block(p: &mut Parser<'_>) -> Result<Block, ParseError> {
    let mut stmts = Vec::new();
    p.skip_newlines();
    let start = p.expect_indent()?;
    let mut end = start;

    while !p.at_eof() {
        if p.peek_kind() == Some(&TokKind::Dedent) {
            end = p.expect_dedent()?;
            break;
        }

        stmts.push(stmt(p)?);
        p.skip_newlines();
    }

    Ok(Block {
        stmts,
        span: start.join(end),
    })
}

fn parse_while(p: &mut Parser<'_>) -> Result<Stmt, ParseError> {
    let start = p.bump().unwrap().span;
    let cond = expr(p, 1)?;
    p.skip_newlines();
    let body = parse_block(p)?;
    Ok(Stmt {
        span: start.join(body.span),
        kind: StmtKind::While { cond, body },
    })
}

fn parse_for(p: &mut Parser<'_>) -> Result<Stmt, ParseError> {
    let start = p.bump().unwrap().span;
    let var = p.expect_ident()?;
    if !p.eat_kw("in") {
        return Err(p.unexpected("in"));
    }
    let iter = expr(p, 1)?;
    p.skip_newlines();
    let body = parse_block(p)?;
    Ok(Stmt {
        span: start.join(body.span),
        kind: StmtKind::For { var, iter, body },
    })
}

fn parse_loop(p: &mut Parser<'_>) -> Result<Stmt, ParseError> {
    let start = p.bump().unwrap().span;
    p.skip_newlines();
    let body = parse_block(p)?;
    Ok(Stmt {
        span: start.join(body.span),
        kind: StmtKind::Loop { body },
    })
}

fn parse_return(p: &mut Parser<'_>) -> Result<Stmt, ParseError> {
    let start = p.bump().unwrap().span;

    // Check if next token is newline or dedent etc
    let is_end = match p.peek_kind() {
        None | Some(TokKind::Eof) | Some(TokKind::Newline) | Some(TokKind::Indent) | Some(TokKind::Dedent) => true,
        _ => false,
    };

    let value = if is_end {
        None
    } else {
        Some(expr(p, 1)?)
    };

    let end = value.as_ref().map(|v| v.span).unwrap_or(start);

    Ok(Stmt {
        span: start.join(end),
        kind: StmtKind::Return(value),
    })
}

fn parse_match(p: &mut Parser<'_>) -> Result<Stmt, ParseError> {
    let start = p.bump().unwrap().span;
    let scrutinee = expr(p, 1)?;
    p.skip_newlines();

    let _indent = p.expect_indent()?;
    let mut arms = Vec::new();
    let mut end = start;

    while !p.at_eof() {
        if p.peek_kind() == Some(&TokKind::Dedent) {
            end = p.expect_dedent()?;
            break;
        }

        let pattern_name = p.expect_ident()?;
        let mut binds = Vec::new();
        let mut p_span = pattern_name.span;

        if p.eat_op(Op::LParen) {
            while !p.check_op(Op::RParen) && !p.at_eof() {
                binds.push(p.expect_ident()?);
            }
            let end_paren = p.expect_op(Op::RParen)?;
            p_span = p_span.join(end_paren);
        }

        p.expect_op(Op::FatArrow)?;
        let arm_body = stmt(p)?;
        arms.push(Arm {
            pattern: Pattern {
                name: pattern_name,
                binds,
                span: p_span,
            },
            span: p_span.join(arm_body.span),
            body: arm_body,
        });

        p.skip_newlines();
    }

    Ok(Stmt {
        span: start.join(end),
        kind: StmtKind::Match { scrutinee, arms },
    })
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Token;
    use crate::diag::Span;

    fn parse(toks: &[Token]) -> Expr {
        let mut p = Parser::new(toks);
        expr(&mut p, 1).unwrap()
    }

    #[test]
    fn test_arithmetic() {
        // 1 + 2 * 3
        let toks = vec![
            Token::int(1),
            Token::op(Op::Plus),
            Token::int(2),
            Token::op(Op::Star),
            Token::int(3),
            Token::eof(),
        ];
        let e = parse(&toks);

        let expected = Expr {
            span: Span::default(),
            kind: ExprKind::Binary {
                op: BinOp::Add,
                lhs: Box::new(Expr {
                    span: Span::default(),
                    kind: ExprKind::Int(1),
                }),
                rhs: Box::new(Expr {
                    span: Span::default(),
                    kind: ExprKind::Binary {
                        op: BinOp::Mul,
                        lhs: Box::new(Expr {
                            span: Span::default(),
                            kind: ExprKind::Int(2),
                        }),
                        rhs: Box::new(Expr {
                            span: Span::default(),
                            kind: ExprKind::Int(3),
                        }),
                    },
                }),
            },
        };

        assert_eq!(e, expected);
    }

    #[test]
    fn test_lambda_and_range() {
        // 1..4
        let toks = vec![
            Token::int(1),
            Token::op(Op::DotDot),
            Token::int(4),
            Token::eof(),
        ];
        let e = parse(&toks);
        assert!(matches!(e.kind, ExprKind::Range { .. }));

        // method chain a.b().c
        let toks = vec![
            Token::ident("a"),
            Token::op(Op::Dot),
            Token::ident("b"),
            Token::op(Op::LParen),
            Token::op(Op::RParen),
            Token::op(Op::Dot),
            Token::ident("c"),
            Token::eof(),
        ];
        let e = parse(&toks);
        if let ExprKind::Field { base, name } = e.kind {
            assert_eq!(name.text, "c");
            if let ExprKind::Call { callee, args: _ } = base.kind {
                if let ExprKind::Field { base, name } = callee.kind {
                    assert_eq!(name.text, "b");
                    if let ExprKind::Name(n) = base.kind {
                        assert_eq!(n.text, "a");
                    } else { panic!("expected Name a"); }
                } else { panic!("expected Field b"); }
            } else { panic!("expected Call"); }
        } else { panic!("expected Field c"); }

    }
    #[test]
    fn test_more() {
        // not a and b
        let toks = vec![
            Token::ident("not"),
            Token::ident("a"),
            Token::ident("and"),
            Token::ident("b"),
            Token::eof(),
        ];
        let e = parse(&toks);
        assert!(matches!(e.kind, ExprKind::Binary { op: BinOp::And, .. }));

        // return x!
        let toks = vec![
            Token::ident("return"),
            Token::ident("x"),
            Token::op(Op::Bang),
            Token::eof(),
        ];
        let mut p = Parser::new(&toks);
        let s = stmt(&mut p).unwrap();
        if let StmtKind::Return(Some(e)) = s.kind {
            assert!(matches!(e.kind, ExprKind::Propagate(_)));
        } else { panic!("expected return"); }

        // list literal [1 2]
        let toks = vec![
            Token::op(Op::LBracket),
            Token::int(1),
            Token::int(2),
            Token::op(Op::RBracket),
            Token::eof(),
        ];
        let e = parse(&toks);
        if let ExprKind::List(items) = e.kind {
            assert_eq!(items.len(), 2);
        } else { panic!("expected List"); }

        // assignment to field
        let toks = vec![
            Token::ident("a"),
            Token::op(Op::Dot),
            Token::ident("b"),
            Token::op(Op::Assign),
            Token::int(1),
            Token::eof(),
        ];
        let mut p = Parser::new(&toks);
        let s = stmt(&mut p).unwrap();
        assert!(matches!(s.kind, StmtKind::Assign { .. }));

        // while cond indent block dedent
        let toks = vec![
            Token::ident("while"),
            Token::ident("a"),
            Token::newline(),
            Token::indent(),
            Token::ident("a"),
            Token::op(Op::Assign),
            Token::int(1),
            Token::dedent(),
            Token::eof(),
        ];
        let mut p = Parser::new(&toks);
        let s = stmt(&mut p).unwrap();
        assert!(matches!(s.kind, StmtKind::While { .. }));
    }
}


fn parse_if(p: &mut Parser<'_>) -> Result<Stmt, ParseError> {
    let start = p.bump().unwrap().span;
    let cond = expr(p, 1)?;

    let then_block = if p.eat_op(Op::FatArrow) {
        let stmt_val = stmt(p)?;
        Block {
            span: stmt_val.span,
            stmts: vec![stmt_val],
        }
    } else {
        p.skip_newlines();
        parse_block(p)?
    };

    let mut elifs = Vec::new();
    while p.eat_kw("elif") {
        let cond = expr(p, 1)?;
        let block = if p.eat_op(Op::FatArrow) {
            let stmt_val = stmt(p)?;
            Block {
                span: stmt_val.span,
                stmts: vec![stmt_val],
            }
        } else {
            p.skip_newlines();
            parse_block(p)?
        };
        elifs.push((cond, block));
    }

    let mut else_ = None;
    if p.eat_kw("else") {
        let block = if p.eat_op(Op::FatArrow) {
            let stmt_val = stmt(p)?;
            Block {
                span: stmt_val.span,
                stmts: vec![stmt_val],
            }
        } else {
            p.skip_newlines();
            parse_block(p)?
        };
        else_ = Some(block);
    }

    let end = else_.as_ref().map(|b| b.span).unwrap_or_else(|| {
        elifs.last().map(|(_, b)| b.span).unwrap_or(then_block.span)
    });

    Ok(Stmt {
        span: start.join(end),
        kind: StmtKind::Expr(Expr {
            span: start.join(end),
            kind: ExprKind::If {
                cond: Box::new(cond),
                then: then_block,
                elifs,
                else_,
            },
        }),
    })
}
