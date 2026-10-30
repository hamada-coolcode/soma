//! Match patterns and arms (owned by task 05).
//!
//! `Pattern` currently models `Name` and `Name(a b)` binds. It also supports wildcard `_`, literals, and enum paths like `Expr.Num` (encoded in `Pattern.name` as `"Expr.Num"`).

use crate::ast::expr::{Arm, Pattern, Stmt};
use crate::diag::Span;
use crate::lexer::{Op, TokKind};

use super::util::Parser;
use super::ParseError;

/// Parse `Pattern => stmt`.
pub fn match_arm(p: &mut Parser<'_>) -> Result<Arm, ParseError> {
    let start = p.peek_span().start;
    let pattern = self::pattern(p)?;
    if !p.eat_op(Op::FatArrow) {
        return Err(ParseError {
            message: "expected `=>` after pattern".to_string(),
            span: p.peek_span(), expected: vec![],
        });
    }
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

    let mut name = match p.peek_kind() {
        Some(TokKind::Ident(_)) => p.expect_ident()?,
        Some(TokKind::Int(v)) => {
            let span = p.peek_span();
            p.pos += 1;
            crate::ast::item::Name::new(v.to_string(), span)
        }
        Some(TokKind::Float(v)) => {
            let span = p.peek_span();
            p.pos += 1;
            crate::ast::item::Name::new(v.to_string(), span)
        }
        Some(TokKind::Str(s)) => {
            let span = p.peek_span();
            let s = s.clone();
            p.pos += 1;
            crate::ast::item::Name::new(format!("\"{}\"", s), span)
        }
        _ => return Err(p.unexpected("pattern")),
    };

    while p.eat_op(Op::Dot) {
        let next_ident = p.expect_ident()?;
        name.text.push('.');
        name.text.push_str(&next_ident.text);
        name.span.end = next_ident.span.end;
    }

    let mut binds = Vec::new();
    if p.eat_op(Op::LParen) {
        while let Some(TokKind::Ident(_)) = p.peek_kind() {
            binds.push(p.expect_ident()?);
        }
        if let Some(TokKind::Op(Op::LParen)) = p.peek_kind() {
            return Err(ParseError {
                message: "nested calls in patterns are not allowed".to_string(),
                span: p.peek_span(), expected: vec![],
            });
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::Token;

    fn parse_pattern(toks: &[Token]) -> Result<Pattern, ParseError> {
        let mut p = Parser::new(toks);
        let pat = pattern(&mut p)?;
        if p.peek_kind().is_some() && p.peek_kind() != Some(&TokKind::Eof) {
            return Err(ParseError {
                message: "trailing".to_string(),
                span: p.peek_span(), expected: vec![],
            });
        }
        Ok(pat)
    }


    #[test]
    fn test_pattern_wildcard() {
        let toks = [Token::ident("_"), Token::eof()];
        let pat = parse_pattern(&toks).unwrap();
        assert_eq!(pat.name.text, "_");
        assert!(pat.binds.is_empty());
    }

    #[test]
    fn test_pattern_literal_int() {
        let toks = [Token::int(42), Token::eof()];
        let pat = parse_pattern(&toks).unwrap();
        assert_eq!(pat.name.text, "42");
        assert!(pat.binds.is_empty());
    }

    #[test]
    fn test_pattern_literal_float() {
        let toks = [Token::float(3.14), Token::eof()];
        let pat = parse_pattern(&toks).unwrap();
        assert_eq!(pat.name.text, "3.14");
        assert!(pat.binds.is_empty());
    }

    #[test]
    fn test_pattern_literal_str() {
        let toks = [Token::str_("hello"), Token::eof()];
        let pat = parse_pattern(&toks).unwrap();
        assert_eq!(pat.name.text, "\"hello\"");
        assert!(pat.binds.is_empty());
    }

    #[test]
    fn test_pattern_dotted() {
        let toks = [
            Token::ident("Expr"),
            Token::op(Op::Dot),
            Token::ident("Num"),
            Token::eof(),
        ];
        let pat = parse_pattern(&toks).unwrap();
        assert_eq!(pat.name.text, "Expr.Num");
        assert!(pat.binds.is_empty());
    }

    #[test]
    fn test_pattern_zero_binds() {
        let toks = [
            Token::ident("None"),
            Token::op(Op::LParen),
            Token::op(Op::RParen),
            Token::eof(),
        ];
        let pat = parse_pattern(&toks).unwrap();
        assert_eq!(pat.name.text, "None");
        assert!(pat.binds.is_empty());
    }

    #[test]
    fn test_pattern_multiple_binds() {
        let toks = [
            Token::ident("Pair"),
            Token::op(Op::LParen),
            Token::ident("a"),
            Token::ident("b"),
            Token::op(Op::RParen),
            Token::eof(),
        ];
        let pat = parse_pattern(&toks).unwrap();
        assert_eq!(pat.name.text, "Pair");
        assert_eq!(pat.binds.len(), 2);
        assert_eq!(pat.binds[0].text, "a");
        assert_eq!(pat.binds[1].text, "b");
    }

    #[test]
    fn test_pattern_nested_rejected() {
        let toks = [
            Token::ident("Expr"),
            Token::op(Op::LParen),
            Token::ident("a"),
            Token::op(Op::LParen),
            Token::ident("b"),
            Token::op(Op::RParen),
            Token::op(Op::RParen),
            Token::eof(),
        ];
        let err = parse_pattern(&toks).unwrap_err();
        assert_eq!(err.message, "nested calls in patterns are not allowed");
    }

    #[test]
    fn test_match_arm_missing_arrow() {
        // `Expr.Num(v) 42` instead of `Expr.Num(v) => 42`
        let toks = [
            Token::ident("Expr"),
            Token::op(Op::Dot),
            Token::ident("Num"),
            Token::op(Op::LParen),
            Token::ident("v"),
            Token::op(Op::RParen),
            Token::int(42),
            Token::eof(),
        ];
        let mut p = Parser::new(&toks);
        let err = match_arm(&mut p).unwrap_err();
        assert_eq!(err.message, "expected `=>` after pattern");
    }

    #[test]
    fn test_match_arm_valid() {
        // `Expr.Num(v) => 42`
        let toks = [
            Token::ident("Expr"),
            Token::op(Op::Dot),
            Token::ident("Num"),
            Token::op(Op::LParen),
            Token::ident("v"),
            Token::op(Op::RParen),
            Token::op(Op::FatArrow),
            Token::int(42),
            Token::eof(),
        ];
        let mut p = Parser::new(&toks);
        let arm = match_arm(&mut p).unwrap();
        assert_eq!(arm.pattern.name.text, "Expr.Num");
        assert_eq!(arm.pattern.binds.len(), 1);
        assert_eq!(arm.pattern.binds[0].text, "v");
    }

    #[test]
    fn test_integration_match_4_arms() {
        // "match e \n  _ => 1\n  Expr.Num(v) => 2\n  \"str\" => 3\n  None() => 4\n"
        let toks = [
            Token::ident("match"),
            Token::ident("e"),
            Token::newline(),
            Token::indent(),
            // arm 1
            Token::ident("_"),
            Token::op(Op::FatArrow),
            Token::int(1),
            Token::newline(),
            // arm 2
            Token::ident("Expr"),
            Token::op(Op::Dot),
            Token::ident("Num"),
            Token::op(Op::LParen),
            Token::ident("v"),
            Token::op(Op::RParen),
            Token::op(Op::FatArrow),
            Token::int(2),
            Token::newline(),
            // arm 3
            Token::str_("str"),
            Token::op(Op::FatArrow),
            Token::int(3),
            Token::newline(),
            // arm 4
            Token::ident("None"),
            Token::op(Op::LParen),
            Token::op(Op::RParen),
            Token::op(Op::FatArrow),
            Token::int(4),
            Token::newline(),
            Token::dedent(),
            Token::eof(),
        ];
        let mut p = Parser::new(&toks);
        let stmt = crate::parse::expr::stmt(&mut p).unwrap();
        if let crate::ast::expr::StmtKind::Match { scrutinee: _, arms } = stmt.kind {
            assert_eq!(arms.len(), 4);
            assert_eq!(arms[0].pattern.name.text, "_");
            assert_eq!(arms[1].pattern.name.text, "Expr.Num");
            assert_eq!(arms[1].pattern.binds.len(), 1);
            assert_eq!(arms[2].pattern.name.text, "\"str\"");
            assert_eq!(arms[3].pattern.name.text, "None");
            assert!(arms[3].pattern.binds.is_empty());
        } else {
            panic!("Expected Match stmt");
        }
    }
}
