//! Match patterns and arms (owned by task 05).
//!
//! `Pattern` currently models `Name` and `Name(a b)` binds. Task 05 extends it
//! to enum paths, literals and `_`; keep `match_arm` and `pattern` signatures.

use crate::ast::expr::{Arm, Pattern, Stmt};
use crate::diag::Span;
use crate::lexer::{Op, TokKind};

use super::ParseError;
use super::util::Parser;

/// Parse `Pattern => stmt`.
pub fn match_arm(p: &mut Parser<'_>) -> Result<Arm, ParseError> {
    let start = p.peek_span().start;
    let pattern = self::pattern(p)?;
    p.expect_op(Op::FatArrow)?;
    let body: Stmt = super::expr::stmt(p)?;
    let end = body.span.end;
    Ok(Arm {
        pattern,
        body,
        span: Span { start, end },
    })
}

/// Parse a single pattern: `Name` or `Name(a b)`.
pub fn pattern(p: &mut Parser<'_>) -> Result<Pattern, ParseError> {
    let start = p.peek_span().start;
    let name = p.expect_ident()?;
    let mut binds = Vec::new();
    if p.eat_op(Op::LParen) {
        while let Some(TokKind::Ident(_)) = p.peek_kind() {
            binds.push(p.expect_ident()?);
        }
        p.expect_op(Op::RParen)?;
    }
    Ok(Pattern {
        name,
        binds,
        span: Span {
            start,
            end: p.toks[p.pos - 1].span.end,
        },
    })
}
