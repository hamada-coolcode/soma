use crate::ast::expr::{BinOp, Block, Expr, ExprKind, Pattern, Stmt, StmtKind, UnOp};
use crate::ast::item::{
    ConstDef, EnumDef, ErrorDef, FnDef, ImportDef, Item, ItemKind, ObjectDef, TestDef, TraitDef,
};
use crate::ast::ty::{TypeKind, TypeRepr};
use crate::lexer::token::FPart;

/// Print the lowered AST as `.psoma`: explicit braces, semicolons, commas.
/// Must be stable and round-trippable for golden-file tests.
pub fn emit(items: &[Item]) -> String {
    let mut out = String::new();
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push_str("\n\n");
        }
        out.push_str(&emit_item(item));
    }
    out.push('\n');
    out
}

pub fn emit_item(item: &Item) -> String {
    emit_item_with_indent(item, 0)
}

fn emit_item_with_indent(item: &Item, indent: usize) -> String {
    let mut out = String::new();
    for attr in &item.attrs {
        out.push_str(&"  ".repeat(indent));
        out.push_str(&format!("#[{}]\n", attr.name.text));
    }
    out.push_str(&"  ".repeat(indent));
    match &item.kind {
        ItemKind::Fn(f) => out.push_str(&emit_fn_def(f, indent)),
        ItemKind::Object(o) => out.push_str(&emit_object_def(o, indent)),
        ItemKind::Trait(t) => out.push_str(&emit_trait_def(t, indent)),
        ItemKind::Error(e) => out.push_str(&emit_error_def(e, indent)),
        ItemKind::Enum(e) => out.push_str(&emit_enum_def(e, indent)),
        ItemKind::Import(i) => out.push_str(&emit_import_def(i, indent)),
        ItemKind::Const(c) => out.push_str(&emit_const_def(c, indent)),
        ItemKind::Test(t) => out.push_str(&emit_test_def(t, indent)),
    }
    out
}

fn emit_fn_def(f: &FnDef, indent: usize) -> String {
    let mut out = String::from("fn ");
    out.push_str(&f.name.text);
    if !f.generics.is_empty() {
        out.push('[');
        let gens: Vec<_> = f.generics.iter().map(|n| n.text.clone()).collect();
        out.push_str(&gens.join(", "));
        out.push(']');
    }
    out.push('(');
    let params: Vec<_> = f
        .params
        .iter()
        .map(|p| format!("{}: {}", p.name.text, emit_ty(&p.ty)))
        .collect();
    out.push_str(&params.join(", "));
    out.push(')');
    if let Some(ret) = &f.ret {
        out.push_str(" -> ");
        out.push_str(&emit_ty(ret));
    }
    out.push(' ');
    out.push_str(&emit_block(&f.body, indent));
    out
}

fn emit_object_def(o: &ObjectDef, indent: usize) -> String {
    let mut out = String::from("object ");
    out.push_str(&o.name.text);
    if !o.generics.is_empty() {
        out.push('[');
        let gens: Vec<_> = o.generics.iter().map(|n| n.text.clone()).collect();
        out.push_str(&gens.join(", "));
        out.push(']');
    }
    out.push_str(" {\n");
    for field in &o.fields {
        out.push_str(&"  ".repeat(indent + 1));
        out.push_str(&format!("{}: {},\n", field.name.text, emit_ty(&field.ty)));
    }
    for method in &o.methods {
        out.push_str(&"  ".repeat(indent + 1));
        out.push_str(&emit_fn_def(method, indent + 1));
        out.push('\n');
    }
    out.push_str(&"  ".repeat(indent));
    out.push('}');
    out
}

fn emit_trait_def(t: &TraitDef, indent: usize) -> String {
    let mut out = String::from("trait ");
    out.push_str(&t.name.text);
    out.push_str(" {\n");
    for method in &t.methods {
        out.push_str(&"  ".repeat(indent + 1));
        out.push_str("fn ");
        out.push_str(&method.name.text);
        if !method.generics.is_empty() {
            out.push('[');
            let gens: Vec<_> = method.generics.iter().map(|n| n.text.clone()).collect();
            out.push_str(&gens.join(", "));
            out.push(']');
        }
        out.push('(');
        let params: Vec<_> = method
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name.text, emit_ty(&p.ty)))
            .collect();
        out.push_str(&params.join(", "));
        out.push(')');
        if let Some(ret) = &method.ret {
            out.push_str(" -> ");
            out.push_str(&emit_ty(ret));
        }
        out.push_str(";\n");
    }
    out.push_str(&"  ".repeat(indent));
    out.push('}');
    out
}

fn emit_error_def(e: &ErrorDef, indent: usize) -> String {
    let mut out = String::from("error ");
    out.push_str(&e.name.text);
    out.push_str(" {\n");
    for variant in &e.variants {
        out.push_str(&"  ".repeat(indent + 1));
        out.push_str(&variant.text);
        out.push_str(",\n");
    }
    out.push_str(&"  ".repeat(indent));
    out.push('}');
    out
}

fn emit_enum_def(e: &EnumDef, indent: usize) -> String {
    let mut out = String::from("enum ");
    out.push_str(&e.name.text);
    out.push_str(" {\n");
    for variant in &e.variants {
        out.push_str(&"  ".repeat(indent + 1));
        out.push_str(&variant.name.text);
        if !variant.payload.is_empty() {
            out.push('(');
            let tys: Vec<_> = variant.payload.iter().map(emit_ty).collect();
            out.push_str(&tys.join(", "));
            out.push(')');
        }
        out.push_str(",\n");
    }
    out.push_str(&"  ".repeat(indent));
    out.push('}');
    out
}

fn emit_import_def(i: &ImportDef, indent: usize) -> String {
    let mut out = String::from("import ");
    let path: Vec<_> = i.path.iter().map(|n| n.text.clone()).collect();
    out.push_str(&path.join("."));
    if !i.children.is_empty() {
        out.push_str(" {\n");
        for child in &i.children {
            out.push_str(&"  ".repeat(indent + 1));
            // Just emit the item.
            let child_str = emit_item_with_indent(child, indent + 1);
            // Replace newlines with indented newlines.
            out.push_str(&child_str.replace('\n', &format!("\n{}", "  ".repeat(indent + 1))));
            out.push('\n');
        }
        out.push_str(&"  ".repeat(indent));
        out.push('}');
    } else {
        out.push(';');
    }
    out
}

fn emit_const_def(c: &ConstDef, indent: usize) -> String {
    let mut out = String::from("const ");
    out.push_str(&c.name.text);
    out.push_str(" = ");
    out.push_str(&emit_expr(&c.value, 0, indent));
    out.push(';');
    out
}

fn emit_test_def(t: &TestDef, indent: usize) -> String {
    let mut out = String::from("test ");
    out.push_str(&format!("\"{}\"", t.name.escape_debug()));
    out.push(' ');
    out.push_str(&emit_block(&t.body, indent));
    out
}

fn emit_ty(ty: &TypeRepr) -> String {
    match &ty.kind {
        TypeKind::Named { name, generics } => {
            let mut out = name.text.clone();
            if !generics.is_empty() {
                out.push('[');
                let gens: Vec<_> = generics.iter().map(emit_ty).collect();
                out.push_str(&gens.join(", "));
                out.push(']');
            }
            out
        }
        TypeKind::Array { elem, size } => {
            let mut out = emit_ty(elem);
            out.push('[');
            if let Some(s) = size {
                out.push_str(&emit_expr(s, 0, 0));
            }
            out.push(']');
            out
        }
        TypeKind::Optional(inner) => {
            format!("{}?", emit_ty(inner))
        }
        TypeKind::Fn { params, ret } => {
            let ps: Vec<_> = params.iter().map(emit_ty).collect();
            format!("({}) {}", ps.join(", "), emit_ty(ret))
        }
        TypeKind::ErrorUnion { err, ok } => {
            format!("{}!{}", emit_ty(err), emit_ty(ok))
        }
        TypeKind::Never => "!".to_string(),
    }
}

fn emit_block(block: &Block, indent: usize) -> String {
    let mut out = String::from("{\n");
    for stmt in &block.stmts {
        out.push_str(&"  ".repeat(indent + 1));
        out.push_str(&emit_stmt(stmt, indent + 1));
        out.push('\n');
    }
    out.push_str(&"  ".repeat(indent));
    out.push('}');
    out
}

fn emit_stmt(stmt: &Stmt, indent: usize) -> String {
    match &stmt.kind {
        StmtKind::Expr(e) => format!("{};", emit_expr(e, 0, indent)),
        StmtKind::Assign { target, value } => {
            format!(
                "{} = {};",
                emit_expr(target, 0, indent),
                emit_expr(value, 0, indent)
            )
        }
        StmtKind::While { cond, body } => {
            format!(
                "while {} {}",
                emit_expr(cond, 0, indent),
                emit_block(body, indent)
            )
        }
        StmtKind::For { var, iter, body } => format!(
            "for {} in {} {}",
            var.text,
            emit_expr(iter, 0, indent),
            emit_block(body, indent)
        ),
        StmtKind::Loop { body } => format!("loop {}", emit_block(body, indent)),
        StmtKind::Break => "break;".to_string(),
        StmtKind::Continue => "continue;".to_string(),
        StmtKind::Return(Some(e)) => format!("return {};", emit_expr(e, 0, indent)),
        StmtKind::Return(None) => "return;".to_string(),
        StmtKind::Match { scrutinee, arms } => {
            let mut out = format!("match {} {{\n", emit_expr(scrutinee, 0, indent));
            for arm in arms {
                out.push_str(&"  ".repeat(indent + 1));
                out.push_str(&emit_pattern(&arm.pattern));
                out.push_str(" => ");
                out.push_str(&emit_stmt(&arm.body, indent + 1));
                out.push('\n');
            }
            out.push_str(&"  ".repeat(indent));
            out.push('}');
            out
        }
        StmtKind::Item(item) => emit_item_with_indent(item, indent),
    }
}

fn emit_pattern(pat: &Pattern) -> String {
    let mut out = pat.name.text.clone();
    if !pat.binds.is_empty() {
        out.push('(');
        let binds: Vec<_> = pat.binds.iter().map(|n| n.text.clone()).collect();
        out.push_str(&binds.join(", "));
        out.push(')');
    }
    out
}

fn emit_expr(expr: &Expr, prec: u8, indent: usize) -> String {
    let (s, op_prec) = match &expr.kind {
        ExprKind::Int(v) => (v.to_string(), 100),
        ExprKind::Float(v) => {
            let mut s = format!("{}", v);
            if !s.contains('.') {
                s.push_str(".0");
            }
            (s, 100)
        }
        ExprKind::Str(s) => (format!("\"{}\"", s.escape_debug()), 100),
        ExprKind::FStr(parts) => {
            let mut out = String::from("f\"");
            for p in parts {
                match p {
                    FPart::Text(t) => out.push_str(&t.escape_debug().to_string()),
                    FPart::Expr(e) => {
                        out.push('{');
                        out.push_str(e);
                        out.push('}');
                    }
                }
            }
            out.push('"');
            (out, 100)
        }
        ExprKind::Bool(b) => (b.to_string(), 100),
        ExprKind::Null => ("null".to_string(), 100),
        ExprKind::Name(n) => (n.text.clone(), 100),
        ExprKind::SelfExpr => ("self".to_string(), 100),
        ExprKind::List(items) => {
            let mut out = String::from("[");
            let emits: Vec<_> = items.iter().map(|e| emit_expr(e, 0, indent)).collect();
            out.push_str(&emits.join(", "));
            out.push(']');
            (out, 100)
        }
        ExprKind::Call { callee, args } => {
            let mut out = emit_expr(callee, 90, indent);
            out.push('(');
            let args_emitted: Vec<_> = args
                .iter()
                .map(|a| {
                    if let Some(n) = &a.name {
                        format!("{} = {}", n.text, emit_expr(&a.value, 0, indent))
                    } else {
                        emit_expr(&a.value, 0, indent)
                    }
                })
                .collect();
            out.push_str(&args_emitted.join(", "));
            out.push(')');
            (out, 90)
        }
        ExprKind::Index { base, index } => {
            let mut out = emit_expr(base, 90, indent);
            out.push('[');
            out.push_str(&emit_expr(index, 0, indent));
            out.push(']');
            (out, 90)
        }
        ExprKind::Field { base, name } => {
            let mut out = emit_expr(base, 90, indent);
            out.push('.');
            out.push_str(&name.text);
            (out, 90)
        }
        ExprKind::Unary { op, operand } => {
            let op_str = match op {
                UnOp::Not => "not ",
                UnOp::Neg => "-",
            };
            (format!("{}{}", op_str, emit_expr(operand, 80, indent)), 80)
        }
        ExprKind::Binary { op, lhs, rhs } => {
            let (op_str, my_prec) = match op {
                BinOp::Mul | BinOp::Div | BinOp::Rem => (
                    match op {
                        BinOp::Mul => " * ",
                        BinOp::Div => " / ",
                        BinOp::Rem => " % ",
                        _ => unreachable!(),
                    },
                    70,
                ),
                BinOp::Add | BinOp::Sub => (
                    match op {
                        BinOp::Add => " + ",
                        BinOp::Sub => " - ",
                        _ => unreachable!(),
                    },
                    60,
                ),
                BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => (
                    match op {
                        BinOp::Lt => " < ",
                        BinOp::Gt => " > ",
                        BinOp::Le => " <= ",
                        BinOp::Ge => " >= ",
                        _ => unreachable!(),
                    },
                    50,
                ),
                BinOp::Eq | BinOp::Ne => (
                    match op {
                        BinOp::Eq => " == ",
                        BinOp::Ne => " != ",
                        _ => unreachable!(),
                    },
                    40,
                ),
                BinOp::And => (" and ", 30),
                BinOp::Or => (" or ", 20),
            };
            // Left-associative: lhs needs same prec, rhs needs higher prec to not group left
            (
                format!(
                    "{}{}{}",
                    emit_expr(lhs, my_prec, indent),
                    op_str,
                    emit_expr(rhs, my_prec + 1, indent)
                ),
                my_prec,
            )
        }
        ExprKind::Range { from, to } => {
            let mut out = String::new();
            if let Some(f) = from {
                out.push_str(&emit_expr(f, 15, indent));
            }
            out.push_str("..");
            if let Some(t) = to {
                out.push_str(&emit_expr(t, 15, indent));
            }
            (out, 10)
        }
        ExprKind::Lambda { params, body } => {
            let ps: Vec<_> = params.iter().map(|n| n.text.clone()).collect();
            (
                format!("|{}| {}", ps.join(", "), emit_expr(body, 0, indent)),
                5,
            )
        }
        ExprKind::Comprehension { expr, var, iter } => (
            format!(
                "[{} for {} in {}]",
                emit_expr(expr, 0, indent),
                var.text,
                emit_expr(iter, 0, indent)
            ),
            100,
        ),
        ExprKind::Propagate(e) => (format!("propagate({})", emit_expr(e, 0, indent)), 100),
        ExprKind::If {
            cond,
            then,
            elifs,
            else_,
        } => {
            let mut out = format!(
                "if {} {}",
                emit_expr(cond, 0, indent),
                emit_block(then, indent)
            );
            for (elif_cond, elif_block) in elifs {
                out.push_str(&format!(
                    " elif {} {}",
                    emit_expr(elif_cond, 0, indent),
                    emit_block(elif_block, indent)
                ));
            }
            if let Some(else_block) = else_ {
                out.push_str(&format!(" else {}", emit_block(else_block, indent)));
            }
            (out, 100)
        }
    };

    if op_prec < prec {
        format!("({})", s)
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::item::{FieldDef, Name, Param};
    use crate::diag::Span;

    fn s() -> Span {
        Span::default()
    }
    fn n(text: &str) -> Name {
        Name::new(text, s())
    }
    fn int_ty() -> TypeRepr {
        TypeRepr {
            kind: TypeKind::Named {
                name: n("int"),
                generics: vec![],
            },
            span: s(),
        }
    }

    #[test]
    fn test_fn_add() {
        let f = FnDef {
            name: n("add"),
            generics: vec![],
            params: vec![
                Param {
                    name: n("a"),
                    ty: int_ty(),
                },
                Param {
                    name: n("b"),
                    ty: int_ty(),
                },
            ],
            ret: Some(int_ty()),
            body: Block {
                stmts: vec![Stmt {
                    kind: StmtKind::Return(Some(Expr {
                        kind: ExprKind::Binary {
                            op: BinOp::Add,
                            lhs: Box::new(Expr {
                                kind: ExprKind::Name(n("a")),
                                span: s(),
                            }),
                            rhs: Box::new(Expr {
                                kind: ExprKind::Name(n("b")),
                                span: s(),
                            }),
                        },
                        span: s(),
                    })),
                    span: s(),
                }],
                span: s(),
            },
        };
        let item = Item {
            attrs: vec![],
            kind: ItemKind::Fn(f),
            span: s(),
        };
        assert_eq!(
            emit_item(&item),
            "fn add(a: int, b: int) -> int {\n  return a + b;\n}"
        );
    }

    #[test]
    fn test_object() {
        let o = ObjectDef {
            name: n("Point"),
            generics: vec![],
            fields: vec![FieldDef {
                name: n("x"),
                ty: int_ty(),
            }],
            methods: vec![FnDef {
                name: n("get_x"),
                generics: vec![],
                params: vec![Param {
                    name: n("self"),
                    ty: int_ty(),
                }], // Using int just for test
                ret: Some(int_ty()),
                body: Block {
                    stmts: vec![Stmt {
                        kind: StmtKind::Return(Some(Expr {
                            kind: ExprKind::Field {
                                base: Box::new(Expr {
                                    kind: ExprKind::SelfExpr,
                                    span: s(),
                                }),
                                name: n("x"),
                            },
                            span: s(),
                        })),
                        span: s(),
                    }],
                    span: s(),
                },
            }],
        };
        let item = Item {
            attrs: vec![],
            kind: ItemKind::Object(o),
            span: s(),
        };
        assert_eq!(
            emit_item(&item),
            "object Point {\n  x: int,\n  fn get_x(self: int) -> int {\n    return self.x;\n  }\n}"
        );
    }

    #[test]
    fn test_if_elif_else() {
        let e = Expr {
            kind: ExprKind::If {
                cond: Box::new(Expr {
                    kind: ExprKind::Name(n("a")),
                    span: s(),
                }),
                then: Block {
                    stmts: vec![Stmt {
                        kind: StmtKind::Expr(Expr {
                            kind: ExprKind::Int(1),
                            span: s(),
                        }),
                        span: s(),
                    }],
                    span: s(),
                },
                elifs: vec![(
                    Expr {
                        kind: ExprKind::Name(n("b")),
                        span: s(),
                    },
                    Block {
                        stmts: vec![Stmt {
                            kind: StmtKind::Expr(Expr {
                                kind: ExprKind::Int(2),
                                span: s(),
                            }),
                            span: s(),
                        }],
                        span: s(),
                    },
                )],
                else_: Some(Block {
                    stmts: vec![Stmt {
                        kind: StmtKind::Expr(Expr {
                            kind: ExprKind::Int(3),
                            span: s(),
                        }),
                        span: s(),
                    }],
                    span: s(),
                }),
            },
            span: s(),
        };
        assert_eq!(
            emit_expr(&e, 0, 0),
            "if a {\n  1;\n} elif b {\n  2;\n} else {\n  3;\n}"
        );
    }

    #[test]
    fn test_match() {
        let stmt = Stmt {
            kind: StmtKind::Match {
                scrutinee: Expr {
                    kind: ExprKind::Name(n("x")),
                    span: s(),
                },
                arms: vec![
                    crate::ast::expr::Arm {
                        pattern: Pattern {
                            name: n("Some"),
                            binds: vec![n("v")],
                            span: s(),
                        },
                        body: Stmt {
                            kind: StmtKind::Expr(Expr {
                                kind: ExprKind::Name(n("v")),
                                span: s(),
                            }),
                            span: s(),
                        },
                        span: s(),
                    },
                    crate::ast::expr::Arm {
                        pattern: Pattern {
                            name: n("None"),
                            binds: vec![],
                            span: s(),
                        },
                        body: Stmt {
                            kind: StmtKind::Expr(Expr {
                                kind: ExprKind::Int(0),
                                span: s(),
                            }),
                            span: s(),
                        },
                        span: s(),
                    },
                ],
            },
            span: s(),
        };
        assert_eq!(
            emit_stmt(&stmt, 0),
            "match x {\n  Some(v) => v;\n  None => 0;\n}"
        );
    }

    #[test]
    fn test_call_args() {
        let e = Expr {
            kind: ExprKind::Call {
                callee: Box::new(Expr {
                    kind: ExprKind::Name(n("f")),
                    span: s(),
                }),
                args: vec![
                    crate::ast::expr::Arg {
                        name: None,
                        value: Expr {
                            kind: ExprKind::Int(1),
                            span: s(),
                        },
                    },
                    crate::ast::expr::Arg {
                        name: Some(n("b")),
                        value: Expr {
                            kind: ExprKind::Int(2),
                            span: s(),
                        },
                    },
                ],
            },
            span: s(),
        };
        assert_eq!(emit_expr(&e, 0, 0), "f(1, b = 2)");
    }

    #[test]
    fn test_list_literal() {
        let e = Expr {
            kind: ExprKind::List(vec![
                Expr {
                    kind: ExprKind::Int(1),
                    span: s(),
                },
                Expr {
                    kind: ExprKind::Int(2),
                    span: s(),
                },
            ]),
            span: s(),
        };
        assert_eq!(emit_expr(&e, 0, 0), "[1, 2]");
    }

    #[test]
    fn test_lambda() {
        let e = Expr {
            kind: ExprKind::Lambda {
                params: vec![n("a"), n("b")],
                body: Box::new(Expr {
                    kind: ExprKind::Binary {
                        op: BinOp::Add,
                        lhs: Box::new(Expr {
                            kind: ExprKind::Name(n("a")),
                            span: s(),
                        }),
                        rhs: Box::new(Expr {
                            kind: ExprKind::Name(n("b")),
                            span: s(),
                        }),
                    },
                    span: s(),
                }),
            },
            span: s(),
        };
        assert_eq!(emit_expr(&e, 0, 0), "|a, b| a + b");
    }
}
