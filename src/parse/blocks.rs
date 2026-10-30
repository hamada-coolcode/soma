use crate::ast::expr::*;
use crate::ast::item::*;
use crate::diag::Span;
use crate::lexer::{Op, TokKind, Token};
use crate::parse::ParseError;
use crate::parse::util::Parser;
use crate::parse::{expr, signature, types};

fn expect_newline(p: &mut Parser) -> Result<Span, ParseError> {
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
fn parse_attrs(p: &mut Parser) -> Result<Vec<Attribute>, ParseError> {
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

fn last_span(p: &Parser) -> Span {
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
        let mut imports = Vec::new();
        expect_newline(p)?;
        p.expect_indent()?;
        while !matches!(p.peek_kind(), Some(TokKind::Dedent)) {
            parse_import_path_node(p, &mut imports, Vec::new(), &attrs, start_span)?;
            if matches!(p.peek_kind(), Some(TokKind::Newline)) {
                expect_newline(p)?;
            }
        }
        p.expect_dedent()?;
        return Ok(imports);
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
    } else if p.eat_kw("object") {
        // One `object` block may declare several objects:
        //   object
        //     Circle
        //       radius: float
        //     Rectangle
        //       ...
        expect_newline(p)?;
        p.expect_indent()?;
        let mut objs = Vec::new();
        while !matches!(p.peek_kind(), Some(TokKind::Dedent)) && !p.at_eof() {
            let obj_attrs = parse_attrs(p)?;
            let obj_start = p.peek_span();
            let name = p.expect_ident()?;
            let mut generics = Vec::new();
            if p.eat_op(Op::LBracket) {
                loop {
                    generics.push(p.expect_ident()?);
                    if p.check_op(Op::RBracket) {
                        break;
                    }
                }
                p.expect_op(Op::RBracket)?;
            }
            expect_newline(p)?;
            p.expect_indent()?;
            let mut fields = Vec::new();
            let mut methods = Vec::new();

            while !matches!(p.peek_kind(), Some(TokKind::Dedent)) && !p.at_eof() {
                let member_name = p.expect_ident()?;
                if p.eat_op(Op::Colon) {
                    // field
                    let ty = types::type_repr(p)?;
                    fields.push(FieldDef {
                        name: member_name,
                        ty,
                    });
                    if matches!(p.peek_kind(), Some(TokKind::Newline)) {
                        expect_newline(p)?;
                    }
                } else {
                    // method
                    let sig;
                    if matches!(p.peek_kind(), Some(TokKind::Newline)) {
                        sig = crate::parse::signature::Signature {
                            name: member_name,
                            generics: Vec::new(),
                            params: Vec::new(),
                            ret: None,
                        };
                    } else {
                        let mut temp_p = Parser {
                            toks: p.toks,
                            pos: p.pos - 1,
                            allow_lambda: true,
                        };
                        sig = crate::parse::signature::signature(&mut temp_p)?;
                        p.pos = temp_p.pos;
                    }

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
                    methods.push(FnDef {
                        name: sig.name,
                        generics: sig.generics,
                        params: sig.params,
                        ret: sig.ret,
                        body: Block {
                            stmts,
                            span: Default::default(),
                        },
                    });
                }
            }
            p.expect_dedent()?;
            objs.push(Item {
                attrs: attrs.iter().cloned().chain(obj_attrs).collect(),
                kind: ItemKind::Object(ObjectDef {
                    name,
                    generics,
                    fields,
                    methods,
                }),
                span: obj_start.join(last_span(p)),
            });
        }
        p.expect_dedent()?;
        return Ok(objs);
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
    } else if p.eat_kw("trait") {
        expect_newline(p)?;
        p.expect_indent()?;
        let name = p.expect_ident()?;
        expect_newline(p)?;
        p.expect_indent()?;
        let mut methods = Vec::new();
        while !matches!(p.peek_kind(), Some(TokKind::Dedent)) {
            let _m_name = p.expect_ident()?;
            let mut temp_p = Parser {
                toks: p.toks,
                pos: p.pos - 1,
                allow_lambda: true,
            };
            let sig = signature::signature(&mut temp_p)?;
            p.pos = temp_p.pos;
            methods.push(TraitMethod {
                name: sig.name,
                generics: sig.generics,
                params: sig.params,
                ret: sig.ret,
            });
            if matches!(p.peek_kind(), Some(TokKind::Newline)) {
                expect_newline(p)?;
            }
        }
        p.expect_dedent()?;
        p.expect_dedent()?;
        kind = ItemKind::Trait(TraitDef { name, methods });
    } else if p.eat_kw("enum") {
        expect_newline(p)?;
        p.expect_indent()?;
        let name = p.expect_ident()?;
        expect_newline(p)?;
        p.expect_indent()?;
        let mut variants = Vec::new();
        while !matches!(p.peek_kind(), Some(TokKind::Dedent)) {
            let v_name = p.expect_ident()?;
            let mut payload = Vec::new();
            if p.eat_op(Op::LParen) {
                payload.push(types::type_repr(p)?);
                while !p.check_op(Op::RParen) {
                    payload.push(types::type_repr(p)?);
                }
                p.expect_op(Op::RParen)?;
            }
            variants.push(EnumVariant {
                name: v_name,
                payload,
            });
            if matches!(p.peek_kind(), Some(TokKind::Newline)) {
                expect_newline(p)?;
            }
        }
        p.expect_dedent()?;
        p.expect_dedent()?;
        kind = ItemKind::Enum(EnumDef { name, variants });
    } else if p.eat_kw("const") {
        expect_newline(p)?;
        p.expect_indent()?;
        let mut consts = Vec::new();
        while !matches!(p.peek_kind(), Some(TokKind::Dedent)) {
            let name = p.expect_ident()?;
            p.expect_op(Op::Assign)?;
            let value = expr::expr(p, 0)?;
            consts.push(Item {
                attrs: attrs.clone(),
                kind: ItemKind::Const(ConstDef { name, value }),
                span: start_span.join(last_span(p)),
            });
            if matches!(p.peek_kind(), Some(TokKind::Newline)) {
                expect_newline(p)?;
            }
        }
        p.expect_dedent()?;
        return Ok(consts);
    } else if p.eat_kw("test") {
        expect_newline(p)?;
        p.expect_indent()?;
        let mut tests = Vec::new();
        while !matches!(p.peek_kind(), Some(TokKind::Dedent)) {
            let name_str = match p.peek_kind() {
                Some(TokKind::Str(s)) => {
                    let s = s.clone();
                    p.pos += 1;
                    s
                }
                _ => return Err(p.unexpected("string literal for test name")),
            };
            expect_newline(p)?;
            p.expect_indent()?;
            let mut stmts = Vec::new();
            while !matches!(p.peek_kind(), Some(TokKind::Dedent)) {
                stmts.push(expr::stmt(p)?);
                if matches!(p.peek_kind(), Some(TokKind::Newline)) {
                    expect_newline(p)?;
                }
            }
            p.expect_dedent()?;
            tests.push(Item {
                attrs: attrs.clone(),
                kind: ItemKind::Test(TestDef {
                    name: name_str,
                    body: Block {
                        stmts,
                        span: Default::default(),
                    },
                }),
                span: start_span.join(last_span(p)),
            });
            if matches!(p.peek_kind(), Some(TokKind::Newline)) {
                expect_newline(p)?;
            }
        }
        p.expect_dedent()?;
        return Ok(tests);
    } else {
        return Err(p.unexpected("item"));
    }

    Ok(vec![Item {
        attrs,
        kind,
        span: start_span.join(last_span(p)),
    }])
}

fn parse_import_path_node(
    p: &mut Parser,
    out: &mut Vec<Item>,
    mut current_path: Vec<Name>,
    attrs: &[Attribute],
    start_span: Span,
) -> Result<(), ParseError> {
    let mut name = p.expect_ident()?;
    if p.eat_op(Op::Colon) {
        let second = p.expect_ident()?;
        name.text.push(':');
        name.text.push_str(&second.text);
        name.span = name.span.join(second.span);
    }
    current_path.push(name);

    let mut children = Vec::new();
    let mut temp_p = Parser {
        toks: p.toks,
        pos: p.pos,
        allow_lambda: true,
    };
    while matches!(temp_p.peek_kind(), Some(TokKind::Newline)) {
        temp_p.pos += 1;
    }
    if matches!(temp_p.peek_kind(), Some(TokKind::Indent)) {
        // Nested
        while matches!(p.peek_kind(), Some(TokKind::Newline)) {
            expect_newline(p)?;
        }
        p.expect_indent()?;
        let mut child_items = Vec::new();
        while !matches!(p.peek_kind(), Some(TokKind::Dedent)) {
            parse_import_path_node(p, &mut child_items, current_path.clone(), attrs, start_span)?;
            if matches!(p.peek_kind(), Some(TokKind::Newline)) {
                expect_newline(p)?;
            }
        }
        p.expect_dedent()?;
        children = child_items;
    }

    out.push(Item {
        attrs: attrs.to_vec(),
        kind: ItemKind::Import(ImportDef {
            path: current_path,
            children,
        }),
        span: start_span,
    });

    Ok(())
}
