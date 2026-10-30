use crate::ast::expr::*;
use crate::ast::item::*;
use crate::diag::Span;
use crate::lexer::{Op, TokKind, Token};
use crate::parse::ParseError;
use crate::parse::util::Parser;
use crate::parse::{expr, signature};

pub(crate) fn expect_newline(p: &mut Parser) -> Result<Span, ParseError> {
    if matches!(p.peek_kind(), Some(TokKind::Newline)) {
        let mut span = p.peek_span();
        p.pos += 1;
        while matches!(p.peek_kind(), Some(TokKind::Newline)) {
            span = span.join(p.peek_span());
            p.pos += 1;
        }
        Ok(span)
    } else {
        Err(p.unexpected("newline"))
    }
}

/// Parse zero or more `#[name]` attributes, skipping the newlines between them.
pub(crate) fn parse_attrs(p: &mut Parser) -> Result<Vec<Attribute>, ParseError> {
    let mut attrs = Vec::new();
    while p.eat_op(Op::AttrOpen) {
        loop {
            attrs.push(Attribute {
                name: p.expect_ident()?,
            });
            if p.check_op(Op::RBracket) {
                break;
            }
        }
        p.expect_op(Op::RBracket)?;
        while matches!(p.peek_kind(), Some(TokKind::Newline)) {
            p.pos += 1;
        }
    }
    Ok(attrs)
}

pub fn parse_file(toks: &[Token]) -> Result<Vec<Item>, ParseError> {
    let mut p = Parser::new(toks);
    let mut items = Vec::new();

    p.skip_newlines();
    while !p.at_eof() {
        let parsed = parse_item(&mut p)?;
        items.extend(parsed);
        p.skip_newlines();
    }

    Ok(items)
}

pub(crate) fn last_span(p: &Parser) -> Span {
    if p.pos > 0 && p.pos <= p.toks.len() {
        p.toks[p.pos - 1].span
    } else {
        Default::default()
    }
}

fn parse_item(p: &mut Parser) -> Result<Vec<Item>, ParseError> {
    let mut attrs = Vec::new();
    let start_span = p.peek_span();
    while p.eat_op(Op::AttrOpen) {
        // `#[`
        attrs.push(Attribute {
            name: p.expect_ident()?,
        });
        while let Ok(name) = p.expect_ident() {
            attrs.push(Attribute { name });
        }
        p.expect_op(Op::RBracket)?;
        p.skip_newlines();
    }

    let _has_export = p.eat_kw("export");

    if p.eat_kw("import") {
        return crate::parse::items::parse_import(p, attrs, start_span);
    }

    if p.eat_kw("object") {
        return crate::parse::items::parse_object(p, attrs, start_span);
    }

    if p.eat_kw("trait") {
        return crate::parse::items::parse_trait(p, attrs, start_span);
    }

    if p.eat_kw("enum") {
        return crate::parse::items::parse_enum(p, attrs, start_span);
    }

    if p.eat_kw("const") {
        return crate::parse::items::parse_const(p, attrs, start_span);
    }

    if p.eat_kw("test") {
        return crate::parse::items::parse_test(p, attrs, start_span);
    }

    let kind;
    if p.eat_kw("error") {
        expect_newline(p)?;
        p.expect_indent()?;
        let name = p.expect_ident()?;
        expect_newline(p)?;
        p.expect_indent()?;
        let mut variants = Vec::new();
        while !matches!(p.peek_kind(), Some(TokKind::Dedent)) {
            variants.push(p.expect_ident()?);
            if matches!(p.peek_kind(), Some(TokKind::Newline)) {
                expect_newline(p)?;
            }
        }
        p.expect_dedent()?;
        p.expect_dedent()?;
        kind = ItemKind::Error(ErrorDef { name, variants });
    } else if p.eat_kw("fn") {
        // One `fn` block may declare several functions:
        //   fn
        //     a x: int -> int
        //       ...
        //     b -> string
        //       ...
        expect_newline(p)?;
        p.expect_indent()?;
        let mut fns = Vec::new();
        while !matches!(p.peek_kind(), Some(TokKind::Dedent)) && !p.at_eof() {
            let sig_attrs = parse_attrs(p)?;
            let sig = signature::signature(p)?;
            expect_newline(p)?;
            p.expect_indent()?;
            let mut stmts = Vec::new();
            while !matches!(p.peek_kind(), Some(TokKind::Dedent)) && !p.at_eof() {
                stmts.push(expr::stmt(p)?);
                if matches!(p.peek_kind(), Some(TokKind::Newline)) {
                    expect_newline(p)?;
                }
            }
            p.expect_dedent()?;
            fns.push(Item {
                attrs: attrs.iter().cloned().chain(sig_attrs).collect(),
                kind: ItemKind::Fn(FnDef {
                    name: sig.name,
                    generics: sig.generics,
                    params: sig.params,
                    ret: sig.ret,
                    body: Block {
                        stmts,
                        span: Default::default(),
                    },
                }),
                span: start_span.join(last_span(p)),
            });
        }
        p.expect_dedent()?;
        return Ok(fns);
    } else {
        return Err(p.unexpected("item"));
    }

    Ok(vec![Item {
        attrs,
        kind,
        span: start_span.join(last_span(p)),
    }])
}
