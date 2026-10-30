use crate::ast::expr::{Arg, Arm, BinOp, Block, Expr, ExprKind, Stmt, StmtKind, UnOp};
use crate::diag::Span;
use crate::lexer::{Op, TokKind};

use super::ParseError;
use super::util::Parser;

/// True when the next non-newline token is the keyword `kw`. Newlines are
/// consumed, because `elif`/`else` sit on their own line after an `=>` body.
fn kw_after_newlines(p: &mut Parser<'_>, kw: &str) -> bool {
    let mut pos = p.pos;
    while matches!(p.toks.get(pos).map(|t| &t.kind), Some(TokKind::Newline)) {
        pos += 1;
    }
    let hit = matches!(p.toks.get(pos).map(|t| &t.kind), Some(TokKind::Ident(s)) if s == kw);
    if hit {
        p.pos = pos;
    }
    hit
}

fn is_lambda_start(p: &Parser<'_>) -> bool {
    let mut pos = p.pos;
    if let Some(TokKind::Ident(_)) = p.toks.get(pos).map(|t| &t.kind) {
        if let Some(TokKind::Op(Op::FatArrow)) = p.toks.get(pos + 1).map(|t| &t.kind) {
            return true;
        }
    }
    if let Some(TokKind::Op(Op::LParen)) = p.toks.get(pos).map(|t| &t.kind) {
        pos += 1;
        while let Some(TokKind::Ident(_)) = p.toks.get(pos).map(|t| &t.kind) {
            pos += 1;
        }
        if let Some(TokKind::Op(Op::RParen)) = p.toks.get(pos).map(|t| &t.kind) {
            if let Some(TokKind::Op(Op::FatArrow)) = p.toks.get(pos + 1).map(|t| &t.kind) {
                return true;
            }
        }
    }
    false
}

fn postfix_binding_power(op: &TokKind) -> Option<u8> {
    match op {
        TokKind::Op(Op::LParen) => Some(11),
        TokKind::Op(Op::LBracket) => Some(11),
        TokKind::Op(Op::Dot) => Some(11),
        TokKind::Op(Op::Bang) => Some(11),
        _ => None,
    }
}

fn infix_binding_power(op: &TokKind) -> Option<(u8, u8)> {
    match op {
        TokKind::Ident(s) if s == "or" => Some((2, 3)),
        TokKind::Ident(s) if s == "and" => Some((3, 4)),
        TokKind::Op(Op::DotDot) => Some((5, 6)),
        TokKind::Op(Op::EqEq) | TokKind::Op(Op::BangEq) => Some((6, 7)),
        TokKind::Op(Op::Lt)
        | TokKind::Op(Op::Gt)
        | TokKind::Op(Op::LtEq)
        | TokKind::Op(Op::GtEq) => Some((7, 8)),
        TokKind::Op(Op::Plus) | TokKind::Op(Op::Minus) => Some((8, 9)),
        TokKind::Op(Op::Star) | TokKind::Op(Op::Slash) | TokKind::Op(Op::Percent) => Some((9, 10)),
        _ => None,
    }
}

fn binop_from_tok(op: &TokKind) -> Option<BinOp> {
    match op {
        TokKind::Ident(s) if s == "or" => Some(BinOp::Or),
        TokKind::Ident(s) if s == "and" => Some(BinOp::And),
        TokKind::Op(Op::EqEq) => Some(BinOp::Eq),
        TokKind::Op(Op::BangEq) => Some(BinOp::Ne),
        TokKind::Op(Op::Lt) => Some(BinOp::Lt),
        TokKind::Op(Op::Gt) => Some(BinOp::Gt),
        TokKind::Op(Op::LtEq) => Some(BinOp::Le),
        TokKind::Op(Op::GtEq) => Some(BinOp::Ge),
        TokKind::Op(Op::Plus) => Some(BinOp::Add),
        TokKind::Op(Op::Minus) => Some(BinOp::Sub),
        TokKind::Op(Op::Star) => Some(BinOp::Mul),
        TokKind::Op(Op::Slash) => Some(BinOp::Div),
        TokKind::Op(Op::Percent) => Some(BinOp::Rem),
        _ => None,
    }
}

pub fn expr(p: &mut Parser<'_>, min_prec: u8) -> Result<Expr, ParseError> {
    let mut lhs = parse_prefix(p)?;

    while let Some(op_kind) = p.peek_kind() {
        if let Some(l_prec) = postfix_binding_power(op_kind) {
            if l_prec < min_prec {
                break;
            }
            // A `(` that opens `(params) =>` belongs to a lambda argument,
            // not to a call on `lhs`.
            if matches!(op_kind, TokKind::Op(Op::LParen)) && is_lambda_start(p) {
                break;
            }
            lhs = parse_postfix(p, lhs)?;
            continue;
        }

        if let Some((l_prec, r_prec)) = infix_binding_power(op_kind) {
            if l_prec < min_prec {
                break;
            }
            let op = p.bump().unwrap().kind.clone();

            if op == TokKind::Op(Op::DotDot) {
                let to = expr(p, r_prec)?;
                let start_span = lhs.span;
                let end_span = to.span;
                lhs = Expr {
                    kind: ExprKind::Range {
                        from: Some(Box::new(lhs)),
                        to: Some(Box::new(to)),
                    },
                    span: Span {
                        start: start_span.start,
                        end: end_span.end,
                    },
                };
            } else if let Some(binop) = binop_from_tok(&op) {
                let rhs = expr(p, r_prec)?;
                let start_span = lhs.span;
                let end_span = rhs.span;
                lhs = Expr {
                    kind: ExprKind::Binary {
                        op: binop,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    span: Span {
                        start: start_span.start,
                        end: end_span.end,
                    },
                };
            } else {
                unreachable!()
            }
            continue;
        }

        break;
    }

    Ok(lhs)
}

fn parse_prefix(p: &mut Parser<'_>) -> Result<Expr, ParseError> {
    if p.allow_lambda && is_lambda_start(p) {
        let start_span = p.peek_span();
        let mut params = Vec::new();

        if p.eat_op(Op::LParen) {
            while let Some(TokKind::Ident(_)) = p.peek_kind() {
                params.push(p.expect_ident()?);
            }
            p.expect_op(Op::RParen)?;
        } else {
            params.push(p.expect_ident()?);
        }

        p.expect_op(Op::FatArrow)?;

        let body = expr(p, 1)?; // lambda right-assoc
        let end_span = body.span;
        return Ok(Expr {
            kind: ExprKind::Lambda {
                params,
                body: Box::new(body),
            },
            span: Span {
                start: start_span.start,
                end: end_span.end,
            },
        });
    }

    if p.at_kw("not") {
        let span = p.peek_span();
        p.bump();
        let operand = expr(p, 4)?;
        let end_span = operand.span;
        return Ok(Expr {
            kind: ExprKind::Unary {
                op: UnOp::Not,
                operand: Box::new(operand),
            },
            span: Span {
                start: span.start,
                end: end_span.end,
            },
        });
    }

    if p.check_op(Op::DotDot) {
        let span = p.peek_span();
        p.bump();
        let to = expr(p, 5)?;
        let end_span = to.span;
        return Ok(Expr {
            kind: ExprKind::Range {
                from: None,
                to: Some(Box::new(to)),
            },
            span: Span {
                start: span.start,
                end: end_span.end,
            },
        });
    }

    if p.check_op(Op::Minus) {
        let span = p.peek_span();
        p.bump();
        let operand = expr(p, 10)?;
        let end_span = operand.span;
        return Ok(Expr {
            kind: ExprKind::Unary {
                op: UnOp::Neg,
                operand: Box::new(operand),
            },
            span: Span {
                start: span.start,
                end: end_span.end,
            },
        });
    }

    parse_primary(p)
}

fn parse_primary(p: &mut Parser<'_>) -> Result<Expr, ParseError> {
    let tok = p
        .peek()
        .cloned()
        .ok_or_else(|| p.unexpected("expression"))?;

    match &tok.kind {
        TokKind::Int(v) => {
            p.bump();
            Ok(Expr {
                kind: ExprKind::Int(*v),
                span: tok.span,
            })
        }
        TokKind::Float(v) => {
            p.bump();
            Ok(Expr {
                kind: ExprKind::Float(*v),
                span: tok.span,
            })
        }
        TokKind::Str(s) => {
            p.bump();
            Ok(Expr {
                kind: ExprKind::Str(s.clone()),
                span: tok.span,
            })
        }
        TokKind::FStr(parts) => {
            p.bump();
            Ok(Expr {
                kind: ExprKind::FStr(parts.clone()),
                span: tok.span,
            })
        }
        TokKind::Ident(s) if s == "true" => {
            p.bump();
            Ok(Expr {
                kind: ExprKind::Bool(true),
                span: tok.span,
            })
        }
        TokKind::Ident(s) if s == "false" => {
            p.bump();
            Ok(Expr {
                kind: ExprKind::Bool(false),
                span: tok.span,
            })
        }
        TokKind::Ident(s) if s == "null" => {
            p.bump();
            Ok(Expr {
                kind: ExprKind::Null,
                span: tok.span,
            })
        }
        TokKind::Ident(s) if s == "self" => {
            p.bump();
            Ok(Expr {
                kind: ExprKind::SelfExpr,
                span: tok.span,
            })
        }
        TokKind::Ident(s) if s == "if" => {
            let start_span = tok.span;
            p.bump();
            parse_if_expr_or_stmt(p, start_span)
        }
        TokKind::Ident(_) => {
            let name = p.expect_ident()?;
            Ok(Expr {
                kind: ExprKind::Name(name.clone()),
                span: name.span,
            })
        }
        TokKind::Op(Op::LParen) => {
            let start = tok.span;
            p.bump();
            let e = expr(p, 0)?;
            let rparen_span = p.expect_op(Op::RParen)?;
            Ok(Expr {
                kind: e.kind,
                span: Span {
                    start: start.start,
                    end: rparen_span.end,
                },
            })
        }
        TokKind::Op(Op::LBracket) => {
            let start = tok.span;
            p.bump();

            if p.check_op(Op::RBracket) {
                let end = p.expect_op(Op::RBracket)?;
                return Ok(Expr {
                    kind: ExprKind::List(vec![]),
                    span: Span {
                        start: start.start,
                        end: end.end,
                    },
                });
            }

            let first = expr(p, 0)?;
            if p.at_kw("for") {
                p.bump();
                let var = p.expect_ident()?;
                if !p.eat_kw("in") {
                    return Err(p.unexpected("'in'"));
                }
                let iter = expr(p, 0)?;
                let end = p.expect_op(Op::RBracket)?;
                Ok(Expr {
                    kind: ExprKind::Comprehension {
                        expr: Box::new(first),
                        var,
                        iter: Box::new(iter),
                    },
                    span: Span {
                        start: start.start,
                        end: end.end,
                    },
                })
            } else {
                let mut list = vec![first];
                while !p.check_op(Op::RBracket) && !p.at_eof() {
                    list.push(expr(p, 0)?);
                }
                let end = p.expect_op(Op::RBracket)?;
                Ok(Expr {
                    kind: ExprKind::List(list),
                    span: Span {
                        start: start.start,
                        end: end.end,
                    },
                })
            }
        }
        _ => Err(p.unexpected("expression")),
    }
}

fn parse_if_expr_or_stmt(p: &mut Parser<'_>, start_span: Span) -> Result<Expr, ParseError> {
    let saved = p.allow_lambda;
    p.allow_lambda = false;
    let cond = expr(p, 0);
    p.allow_lambda = saved;
    let cond = cond?;
    let then = parse_if_body(p)?;

    let mut elifs = Vec::new();
    while kw_after_newlines(p, "elif") {
        p.pos += 1;
        let saved = p.allow_lambda;
        p.allow_lambda = false;
        let elif_cond = expr(p, 0);
        p.allow_lambda = saved;
        let elif_cond = elif_cond?;
        let elif_body = parse_if_body(p)?;
        elifs.push((elif_cond, elif_body));
    }

    let mut else_ = None;
    if kw_after_newlines(p, "else") {
        p.pos += 1;
        else_ = Some(parse_if_body(p)?);
    }

    let mut end = then.span.end;
    if let Some(e) = &else_ {
        end = e.span.end;
    } else if let Some((_, b)) = elifs.last() {
        end = b.span.end;
    }

    Ok(Expr {
        kind: ExprKind::If {
            cond: Box::new(cond),
            then,
            elifs,
            else_,
        },
        span: Span {
            start: start_span.start,
            end,
        },
    })
}

fn parse_if_body(p: &mut Parser<'_>) -> Result<Block, ParseError> {
    if p.eat_op(Op::FatArrow) {
        let stmt = parse_simple_stmt(p)?;
        let span = stmt.span;
        Ok(Block {
            stmts: vec![stmt],
            span,
        })
    } else {
        p.skip_newlines();
        p.expect_indent()?;
        let block = parse_block(p)?;
        p.expect_dedent()?;
        Ok(block)
    }
}

fn parse_postfix(p: &mut Parser<'_>, lhs: Expr) -> Result<Expr, ParseError> {
    // `f(x)` is a call, but `reduce(0 (a b) => a + b)` passes a lambda as the
    // second argument — a `(` that starts `(params) =>` is not a call.
    if p.check_op(Op::LParen) && !is_lambda_start(p) {
        let start = lhs.span.start;
        p.bump();
        let mut args = Vec::new();
        while !p.check_op(Op::RParen) && !p.at_eof() {
            let mut name = None;
            if let Some(TokKind::Ident(_)) = p.peek_kind() {
                if let Some(TokKind::Op(Op::Assign)) = p.toks.get(p.pos + 1).map(|t| &t.kind) {
                    name = Some(p.expect_ident()?);
                    p.bump();
                }
            }
            let saved = p.allow_lambda;
            p.allow_lambda = true;
            let value = expr(p, 0);
            p.allow_lambda = saved;
            args.push(Arg {
                name,
                value: value?,
            });
        }
        let end = p.expect_op(Op::RParen)?;
        Ok(Expr {
            kind: ExprKind::Call {
                callee: Box::new(lhs),
                args,
            },
            span: Span {
                start,
                end: end.end,
            },
        })
    } else if p.check_op(Op::LBracket) {
        let start = lhs.span.start;
        p.bump();
        let index = expr(p, 0)?;
        let end = p.expect_op(Op::RBracket)?;
        Ok(Expr {
            kind: ExprKind::Index {
                base: Box::new(lhs),
                index: Box::new(index),
            },
            span: Span {
                start,
                end: end.end,
            },
        })
    } else if p.check_op(Op::Dot) {
        let start = lhs.span.start;
        p.bump();
        let name = p.expect_ident()?;
        let end = name.span.end;
        Ok(Expr {
            kind: ExprKind::Field {
                base: Box::new(lhs),
                name,
            },
            span: Span { start, end },
        })
    } else if p.check_op(Op::Bang) {
        let start = lhs.span.start;
        let end = p.expect_op(Op::Bang)?;
        Ok(Expr {
            kind: ExprKind::Propagate(Box::new(lhs)),
            span: Span {
                start,
                end: end.end,
            },
        })
    } else {
        unreachable!()
    }
}

pub fn stmt(p: &mut Parser<'_>) -> Result<Stmt, ParseError> {
    let span_start = p.peek_span().start;

    if p.at_kw("while") {
        p.bump();
        let cond = expr(p, 0)?;
        p.skip_newlines();
        p.expect_indent()?;
        let body = parse_block(p)?;
        let end = p.expect_dedent()?.end;
        return Ok(Stmt {
            kind: StmtKind::While { cond, body },
            span: Span {
                start: span_start,
                end,
            },
        });
    }

    if p.at_kw("for") {
        p.bump();
        let var = p.expect_ident()?;
        if !p.eat_kw("in") {
            return Err(p.unexpected("'in'"));
        }
        let iter = expr(p, 0)?;
        p.skip_newlines();
        p.expect_indent()?;
        let body = parse_block(p)?;
        let end = p.expect_dedent()?.end;
        return Ok(Stmt {
            kind: StmtKind::For { var, iter, body },
            span: Span {
                start: span_start,
                end,
            },
        });
    }

    if p.at_kw("loop") {
        p.bump();
        p.skip_newlines();
        p.expect_indent()?;
        let body = parse_block(p)?;
        let end = p.expect_dedent()?.end;
        return Ok(Stmt {
            kind: StmtKind::Loop { body },
            span: Span {
                start: span_start,
                end,
            },
        });
    }

    if p.at_kw("break") {
        let end = p.peek_span().end;
        p.bump();
        return Ok(Stmt {
            kind: StmtKind::Break,
            span: Span {
                start: span_start,
                end,
            },
        });
    }

    if p.at_kw("continue") {
        let end = p.peek_span().end;
        p.bump();
        return Ok(Stmt {
            kind: StmtKind::Continue,
            span: Span {
                start: span_start,
                end,
            },
        });
    }

    if p.at_kw("return") {
        p.bump();
        let mut value = None;
        let mut end = span_start;
        if !matches!(
            p.peek_kind(),
            Some(TokKind::Newline | TokKind::Dedent | TokKind::Eof)
        ) {
            let e = expr(p, 0)?;
            end = e.span.end;
            value = Some(e);
        } else if p.pos > 0 {
            end = p.toks[p.pos - 1].span.end;
        }
        return Ok(Stmt {
            kind: StmtKind::Return(value),
            span: Span {
                start: span_start,
                end,
            },
        });
    }

    if p.at_kw("match") {
        p.bump();
        let scrutinee = expr(p, 0)?;
        p.skip_newlines();
        p.expect_indent()?;
        let mut arms = Vec::new();
        while !matches!(
            p.peek_kind(),
            Some(TokKind::Dedent) | None | Some(TokKind::Eof)
        ) {
            p.skip_newlines();
            if matches!(
                p.peek_kind(),
                Some(TokKind::Dedent) | None | Some(TokKind::Eof)
            ) {
                break;
            }
            arms.push(parse_arm(p)?);
            p.skip_newlines();
        }
        let end = p.expect_dedent()?.end;
        return Ok(Stmt {
            kind: StmtKind::Match { scrutinee, arms },
            span: Span {
                start: span_start,
                end,
            },
        });
    }

    let lhs = expr(p, 0)?;

    if p.eat_op(Op::Assign) {
        let value = expr(p, 0)?;
        let end = value.span.end;
        return Ok(Stmt {
            kind: StmtKind::Assign { target: lhs, value },
            span: Span {
                start: span_start,
                end,
            },
        });
    }

    let end = lhs.span.end;
    Ok(Stmt {
        kind: StmtKind::Expr(lhs),
        span: Span {
            start: span_start,
            end,
        },
    })
}

fn parse_simple_stmt(p: &mut Parser<'_>) -> Result<Stmt, ParseError> {
    let start = p.peek_span().start;
    if p.at_kw("return") {
        p.bump();
        let mut value = None;
        let mut end = start;
        if !matches!(
            p.peek_kind(),
            Some(TokKind::Newline | TokKind::Dedent | TokKind::Eof)
        ) {
            let e = expr(p, 0)?;
            end = e.span.end;
            value = Some(e);
        } else if p.pos > 0 {
            end = p.toks[p.pos - 1].span.end;
        }
        return Ok(Stmt {
            kind: StmtKind::Return(value),
            span: Span { start, end },
        });
    }
    if p.at_kw("break") {
        let end = p.peek_span().end;
        p.bump();
        return Ok(Stmt {
            kind: StmtKind::Break,
            span: Span { start, end },
        });
    }
    if p.at_kw("continue") {
        let end = p.peek_span().end;
        p.bump();
        return Ok(Stmt {
            kind: StmtKind::Continue,
            span: Span { start, end },
        });
    }
    let e = expr(p, 0)?;
    let end = e.span.end;
    Ok(Stmt {
        kind: StmtKind::Expr(e),
        span: Span { start, end },
    })
}

fn parse_block(p: &mut Parser<'_>) -> Result<Block, ParseError> {
    let mut stmts = Vec::new();
    let start = p.peek_span().start;
    let mut end = start;
    while !matches!(
        p.peek_kind(),
        Some(TokKind::Dedent) | None | Some(TokKind::Eof)
    ) {
        p.skip_newlines();
        if matches!(
            p.peek_kind(),
            Some(TokKind::Dedent) | None | Some(TokKind::Eof)
        ) {
            break;
        }
        let s = stmt(p)?;
        end = s.span.end;
        stmts.push(s);
        p.skip_newlines();
    }
    Ok(Block {
        stmts,
        span: Span { start, end },
    })
}

fn parse_arm(p: &mut Parser<'_>) -> Result<Arm, ParseError> {
    crate::parse::pattern::match_arm(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::*;
    use crate::ast::item::Name;
    use crate::lexer::Token;

    fn parse_expr_helper(toks: &[Token]) -> Expr {
        let mut p = Parser::new(toks);
        expr(&mut p, 0).unwrap()
    }

    fn parse_stmt_helper(toks: &[Token]) -> Stmt {
        let mut p = Parser::new(toks);
        stmt(&mut p).unwrap()
    }

    fn n(text: &str) -> Name {
        Name::new(text.to_string(), Span::default())
    }

    fn e_name(text: &str) -> Expr {
        Expr {
            kind: ExprKind::Name(n(text)),
            span: Span::default(),
        }
    }

    fn e_int(v: i64) -> Expr {
        Expr {
            kind: ExprKind::Int(v),
            span: Span::default(),
        }
    }

    #[test]
    fn test_arithmetic_precedence() {
        let toks = [
            Token::int(1),
            Token::op(Op::Plus),
            Token::int(2),
            Token::op(Op::Star),
            Token::int(3),
        ];
        let e = parse_expr_helper(&toks);
        let expected = Expr {
            kind: ExprKind::Binary {
                op: BinOp::Add,
                lhs: Box::new(e_int(1)),
                rhs: Box::new(Expr {
                    kind: ExprKind::Binary {
                        op: BinOp::Mul,
                        lhs: Box::new(e_int(2)),
                        rhs: Box::new(e_int(3)),
                    },
                    span: Span::default(),
                }),
            },
            span: Span::default(),
        };
        assert_eq!(e, expected);
    }

    #[test]
    fn test_unary_minus() {
        let toks = [Token::op(Op::Minus), Token::int(42)];
        let e = parse_expr_helper(&toks);
        assert_eq!(
            e,
            Expr {
                kind: ExprKind::Unary {
                    op: UnOp::Neg,
                    operand: Box::new(e_int(42))
                },
                span: Span::default()
            }
        );
    }

    #[test]
    fn test_not_a_and_b() {
        let toks = [
            Token::ident("not"),
            Token::ident("a"),
            Token::ident("and"),
            Token::ident("b"),
        ];
        let e = parse_expr_helper(&toks);
        assert_eq!(
            e,
            Expr {
                kind: ExprKind::Binary {
                    op: BinOp::And,
                    lhs: Box::new(Expr {
                        kind: ExprKind::Unary {
                            op: UnOp::Not,
                            operand: Box::new(e_name("a"))
                        },
                        span: Span::default()
                    }),
                    rhs: Box::new(e_name("b")),
                },
                span: Span::default()
            }
        );
    }

    #[test]
    fn test_range() {
        let toks = [Token::op(Op::DotDot), Token::int(4)];
        let e = parse_expr_helper(&toks);
        assert_eq!(
            e,
            Expr {
                kind: ExprKind::Range {
                    from: None,
                    to: Some(Box::new(e_int(4)))
                },
                span: Span::default()
            }
        );

        let toks2 = [Token::int(1), Token::op(Op::DotDot), Token::int(4)];
        let e2 = parse_expr_helper(&toks2);
        assert_eq!(
            e2,
            Expr {
                kind: ExprKind::Range {
                    from: Some(Box::new(e_int(1))),
                    to: Some(Box::new(e_int(4)))
                },
                span: Span::default()
            }
        );
    }

    #[test]
    fn test_method_chain() {
        let toks = [
            Token::ident("a"),
            Token::op(Op::Dot),
            Token::ident("b"),
            Token::op(Op::LParen),
            Token::op(Op::RParen),
            Token::op(Op::Dot),
            Token::ident("c"),
        ];
        let e = parse_expr_helper(&toks);
        let field_b = Expr {
            kind: ExprKind::Field {
                base: Box::new(e_name("a")),
                name: n("b"),
            },
            span: Span::default(),
        };
        let call_b = Expr {
            kind: ExprKind::Call {
                callee: Box::new(field_b),
                args: vec![],
            },
            span: Span::default(),
        };
        let field_c = Expr {
            kind: ExprKind::Field {
                base: Box::new(call_b),
                name: n("c"),
            },
            span: Span::default(),
        };
        assert_eq!(e, field_c);
    }

    #[test]
    fn test_index() {
        let toks = [
            Token::ident("a"),
            Token::op(Op::LBracket),
            Token::int(1),
            Token::op(Op::RBracket),
        ];
        let e = parse_expr_helper(&toks);
        assert_eq!(
            e,
            Expr {
                kind: ExprKind::Index {
                    base: Box::new(e_name("a")),
                    index: Box::new(e_int(1))
                },
                span: Span::default()
            }
        );
    }

    #[test]
    fn test_call_named_arg() {
        let toks = [
            Token::ident("f"),
            Token::op(Op::LParen),
            Token::ident("x"),
            Token::op(Op::Assign),
            Token::int(1),
            Token::op(Op::RParen),
        ];
        let e = parse_expr_helper(&toks);
        assert_eq!(
            e,
            Expr {
                kind: ExprKind::Call {
                    callee: Box::new(e_name("f")),
                    args: vec![Arg {
                        name: Some(n("x")),
                        value: e_int(1)
                    }]
                },
                span: Span::default()
            }
        );
    }

    #[test]
    fn test_list_literal() {
        let toks = [
            Token::op(Op::LBracket),
            Token::int(1),
            Token::int(2),
            Token::op(Op::RBracket),
        ];
        let e = parse_expr_helper(&toks);
        assert_eq!(
            e,
            Expr {
                kind: ExprKind::List(vec![e_int(1), e_int(2)]),
                span: Span::default()
            }
        );
    }

    #[test]
    fn test_comprehension() {
        let toks = [
            Token::op(Op::LBracket),
            Token::ident("e"),
            Token::ident("for"),
            Token::ident("x"),
            Token::ident("in"),
            Token::ident("it"),
            Token::op(Op::RBracket),
        ];
        let e = parse_expr_helper(&toks);
        assert_eq!(
            e,
            Expr {
                kind: ExprKind::Comprehension {
                    expr: Box::new(e_name("e")),
                    var: n("x"),
                    iter: Box::new(e_name("it"))
                },
                span: Span::default()
            }
        );
    }

    #[test]
    fn test_lambda() {
        let toks1 = [
            Token::ident("x"),
            Token::op(Op::FatArrow),
            Token::ident("e"),
        ];
        let e1 = parse_expr_helper(&toks1);
        assert_eq!(
            e1,
            Expr {
                kind: ExprKind::Lambda {
                    params: vec![n("x")],
                    body: Box::new(e_name("e"))
                },
                span: Span::default()
            }
        );

        let toks2 = [
            Token::op(Op::LParen),
            Token::ident("x"),
            Token::ident("y"),
            Token::op(Op::RParen),
            Token::op(Op::FatArrow),
            Token::ident("e"),
        ];
        let e2 = parse_expr_helper(&toks2);
        assert_eq!(
            e2,
            Expr {
                kind: ExprKind::Lambda {
                    params: vec![n("x"), n("y")],
                    body: Box::new(e_name("e"))
                },
                span: Span::default()
            }
        );
    }

    #[test]
    fn test_return_x_bang() {
        let toks = [
            Token::ident("return"),
            Token::ident("x"),
            Token::op(Op::Bang),
        ];
        let s = parse_stmt_helper(&toks);
        let x_bang = Expr {
            kind: ExprKind::Propagate(Box::new(e_name("x"))),
            span: Span::default(),
        };
        assert_eq!(
            s,
            Stmt {
                kind: StmtKind::Return(Some(x_bang)),
                span: Span::default()
            }
        );
    }

    #[test]
    fn test_assignment() {
        let toks = [
            Token::ident("x"),
            Token::op(Op::Dot),
            Token::ident("f"),
            Token::op(Op::Assign),
            Token::int(1),
        ];
        let s = parse_stmt_helper(&toks);
        let target = Expr {
            kind: ExprKind::Field {
                base: Box::new(e_name("x")),
                name: n("f"),
            },
            span: Span::default(),
        };
        assert_eq!(
            s,
            Stmt {
                kind: StmtKind::Assign {
                    target,
                    value: e_int(1)
                },
                span: Span::default()
            }
        );

        let toks2 = [
            Token::ident("x"),
            Token::op(Op::LBracket),
            Token::int(0),
            Token::op(Op::RBracket),
            Token::op(Op::Assign),
            Token::int(1),
        ];
        let s2 = parse_stmt_helper(&toks2);
        let target2 = Expr {
            kind: ExprKind::Index {
                base: Box::new(e_name("x")),
                index: Box::new(e_int(0)),
            },
            span: Span::default(),
        };
        assert_eq!(
            s2,
            Stmt {
                kind: StmtKind::Assign {
                    target: target2,
                    value: e_int(1)
                },
                span: Span::default()
            }
        );
    }

    #[test]
    fn test_loops() {
        // while cond NL INDENT block DEDENT
        let toks = [
            Token::ident("while"),
            Token::ident("c"),
            Token::newline(),
            Token::indent(),
            Token::ident("return"),
            Token::dedent(),
        ];
        let s = parse_stmt_helper(&toks);
        let block = Block {
            stmts: vec![Stmt {
                kind: StmtKind::Return(None),
                span: Span::default(),
            }],
            span: Span::default(),
        };
        assert_eq!(
            s,
            Stmt {
                kind: StmtKind::While {
                    cond: e_name("c"),
                    body: block.clone()
                },
                span: Span::default()
            }
        );

        // for x in it NL INDENT block DEDENT
        let toks2 = [
            Token::ident("for"),
            Token::ident("x"),
            Token::ident("in"),
            Token::ident("it"),
            Token::newline(),
            Token::indent(),
            Token::ident("return"),
            Token::dedent(),
        ];
        let s2 = parse_stmt_helper(&toks2);
        assert_eq!(
            s2,
            Stmt {
                kind: StmtKind::For {
                    var: n("x"),
                    iter: e_name("it"),
                    body: block.clone()
                },
                span: Span::default()
            }
        );

        // loop NL INDENT block DEDENT
        let toks3 = [
            Token::ident("loop"),
            Token::newline(),
            Token::indent(),
            Token::ident("return"),
            Token::dedent(),
        ];
        let s3 = parse_stmt_helper(&toks3);
        assert_eq!(
            s3,
            Stmt {
                kind: StmtKind::Loop { body: block },
                span: Span::default()
            }
        );
    }

    #[test]
    fn test_match() {
        let toks = [
            Token::ident("match"),
            Token::ident("x"),
            Token::newline(),
            Token::indent(),
            Token::ident("A"),
            Token::op(Op::FatArrow),
            Token::ident("return"),
            Token::newline(),
            Token::ident("B"),
            Token::op(Op::LParen),
            Token::ident("v"),
            Token::op(Op::RParen),
            Token::op(Op::FatArrow),
            Token::ident("break"),
            Token::dedent(),
        ];
        let s = parse_stmt_helper(&toks);
        let arms = vec![
            Arm {
                pattern: Pattern {
                    name: n("A"),
                    binds: vec![],
                    span: Span::default(),
                },
                body: Stmt {
                    kind: StmtKind::Return(None),
                    span: Span::default(),
                },
                span: Span::default(),
            },
            Arm {
                pattern: Pattern {
                    name: n("B"),
                    binds: vec![n("v")],
                    span: Span::default(),
                },
                body: Stmt {
                    kind: StmtKind::Break,
                    span: Span::default(),
                },
                span: Span::default(),
            },
        ];
        assert_eq!(
            s,
            Stmt {
                kind: StmtKind::Match {
                    scrutinee: e_name("x"),
                    arms
                },
                span: Span::default()
            }
        );
    }
}
