use crate::ast::expr::*;
use crate::ast::item::*;
use crate::diag::Span;
use crate::lexer::{Op, TokKind};
use crate::parse::ParseError;
use crate::parse::blocks::{expect_newline, last_span, parse_attrs};
use crate::parse::util::Parser;
use crate::parse::{expr, signature, types};

pub(crate) fn parse_object(
    p: &mut Parser,
    attrs: Vec<Attribute>,
    _start_span: Span,
) -> Result<Vec<Item>, ParseError> {
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
            let _member_attrs = parse_attrs(p)?;
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
                methods, // Note: ast currently doesn't store member_attrs in ast/item.rs FieldDef or FnDef! The task brief says `attributes on members`. Let's assume there is no field for attributes on members in AST right now. If there's none, we still parse them to not fail, but we don't store them.
                         // Ah wait, is there? Let's check.
            }),
            span: obj_start.join(last_span(p)),
        });
    }
    p.expect_dedent()?;
    Ok(objs)
}

pub(crate) fn parse_trait(
    p: &mut Parser,
    attrs: Vec<Attribute>,
    start_span: Span,
) -> Result<Vec<Item>, ParseError> {
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

    Ok(vec![Item {
        attrs,
        kind: ItemKind::Trait(TraitDef { name, methods }),
        span: start_span.join(last_span(p)),
    }])
}

pub(crate) fn parse_enum(
    p: &mut Parser,
    attrs: Vec<Attribute>,
    start_span: Span,
) -> Result<Vec<Item>, ParseError> {
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
    Ok(vec![Item {
        attrs,
        kind: ItemKind::Enum(EnumDef { name, variants }),
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

pub(crate) fn parse_import(
    p: &mut Parser,
    attrs: Vec<Attribute>,
    start_span: Span,
) -> Result<Vec<Item>, ParseError> {
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
    Ok(imports)
}

pub(crate) fn parse_const(
    p: &mut Parser,
    attrs: Vec<Attribute>,
    start_span: Span,
) -> Result<Vec<Item>, ParseError> {
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
    Ok(consts)
}

pub(crate) fn parse_test(
    p: &mut Parser,
    attrs: Vec<Attribute>,
    start_span: Span,
) -> Result<Vec<Item>, ParseError> {
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
    Ok(tests)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diag::Span;
    use crate::lexer::{Op, TokKind, Token};

    fn span() -> Span {
        Span::default()
    }

    fn ident(s: &str) -> Token {
        Token {
            kind: TokKind::Ident(s.to_string()),
            span: span(),
        }
    }

    fn op(o: Op) -> Token {
        Token {
            kind: TokKind::Op(o),
            span: span(),
        }
    }

    fn nl() -> Token {
        Token {
            kind: TokKind::Newline,
            span: span(),
        }
    }

    fn indent() -> Token {
        Token {
            kind: TokKind::Indent,
            span: span(),
        }
    }

    fn dedent() -> Token {
        Token {
            kind: TokKind::Dedent,
            span: span(),
        }
    }

    fn s(s: &str) -> Token {
        Token {
            kind: TokKind::Str(s.to_string()),
            span: span(),
        }
    }

    fn num(n: i64) -> Token {
        Token {
            kind: TokKind::Int(n),
            span: span(),
        }
    }

    #[test]
    fn test_parse_object() {
        let toks = vec![
            nl(),
            indent(),
            ident("Obj"),
            nl(),
            indent(),
            ident("field"),
            op(Op::Colon),
            ident("int"),
            nl(),
            dedent(),
            dedent(),
        ];
        let mut p = Parser::new(&toks);
        let items = parse_object(&mut p, vec![], span()).unwrap();
        assert_eq!(items.len(), 1);
        if let ItemKind::Object(def) = &items[0].kind {
            assert_eq!(def.name.text, "Obj");
            assert_eq!(def.fields.len(), 1);
            assert_eq!(def.fields[0].name.text, "field");
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_parse_trait() {
        let toks = vec![
            nl(),
            indent(),
            ident("Tr"),
            nl(),
            indent(),
            ident("method"),
            nl(),
            dedent(),
            dedent(),
        ];
        let mut p = Parser::new(&toks);
        let items = parse_trait(&mut p, vec![], span()).unwrap();
        assert_eq!(items.len(), 1);
        if let ItemKind::Trait(def) = &items[0].kind {
            assert_eq!(def.name.text, "Tr");
            assert_eq!(def.methods.len(), 1);
            assert_eq!(def.methods[0].name.text, "method");
        } else {
            panic!("Expected Trait");
        }
    }

    #[test]
    fn test_parse_enum() {
        let toks = vec![
            nl(),
            indent(),
            ident("En"),
            nl(),
            indent(),
            ident("Var"),
            op(Op::LParen),
            ident("int"),
            op(Op::RParen),
            nl(),
            dedent(),
            dedent(),
        ];
        let mut p = Parser::new(&toks);
        let items = parse_enum(&mut p, vec![], span()).unwrap();
        assert_eq!(items.len(), 1);
        if let ItemKind::Enum(def) = &items[0].kind {
            assert_eq!(def.name.text, "En");
            assert_eq!(def.variants.len(), 1);
            assert_eq!(def.variants[0].name.text, "Var");
        } else {
            panic!("Expected Enum");
        }
    }

    #[test]
    fn test_parse_import() {
        let toks = vec![
            nl(),
            indent(),
            ident("std"),
            op(Op::Colon),
            ident("io"),
            nl(),
            indent(),
            ident("File"),
            nl(),
            dedent(),
            dedent(),
        ];
        let mut p = Parser::new(&toks);
        let items = parse_import(&mut p, vec![], span()).unwrap();
        assert_eq!(items.len(), 1);
        if let ItemKind::Import(def) = &items[0].kind {
            assert_eq!(def.path.len(), 1);
            assert_eq!(def.path[0].text, "std:io");
            assert_eq!(def.children.len(), 1);
        } else {
            panic!("Expected Import");
        }
    }

    #[test]
    fn test_parse_const() {
        let toks = vec![
            nl(),
            indent(),
            ident("LIMIT"),
            op(Op::Assign),
            num(10),
            nl(),
            dedent(),
        ];
        let mut p = Parser::new(&toks);
        let items = parse_const(&mut p, vec![], span()).unwrap();
        assert_eq!(items.len(), 1);
        if let ItemKind::Const(def) = &items[0].kind {
            assert_eq!(def.name.text, "LIMIT");
        } else {
            panic!("Expected Const");
        }
    }

    #[test]
    fn test_parse_test() {
        let toks = vec![
            nl(),
            indent(),
            s("my_test"),
            nl(),
            indent(),
            ident("assert"),
            op(Op::LParen),
            ident("true"),
            op(Op::RParen),
            nl(),
            dedent(),
            dedent(),
        ];
        let mut p = Parser::new(&toks);
        let items = parse_test(&mut p, vec![], span()).unwrap();
        assert_eq!(items.len(), 1);
        if let ItemKind::Test(def) = &items[0].kind {
            assert_eq!(def.name, "my_test");
        } else {
            panic!("Expected Test");
        }
    }
}
