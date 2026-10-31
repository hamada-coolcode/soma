use crate::ast::expr::{BinOp, Block, Expr, ExprKind, Pattern, Stmt, StmtKind, UnOp};
use crate::ast::item::{
    ConstDef, EnumDef, ErrorDef, FnDef, ImportDef, Item, ItemKind, ObjectDef, TestDef, TraitDef,
};
use crate::ast::ty::{TypeKind, TypeRepr};
use crate::lexer::token::FPart;
use crate::diag::Span;

pub struct Emitter {
    pub text: String,
    pub map: Vec<(usize, Span)>,
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            map: Vec::new(),
        }
    }

    pub fn raw(&mut self, s: &str) {
        self.text.push_str(s);
    }

    pub fn mark(&mut self, span: Span) {
        if !span.is_empty() {
            self.map.push((self.text.len(), span));
        }
    }
}

pub fn emit_with_map(items: &[Item]) -> (String, Vec<(usize, Span)>) {
    let mut em = Emitter::new();
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            em.raw("\n\n");
        }
        emit_item_impl(item, &mut em, 0);
    }
    em.raw("\n");
    (em.text, em.map)
}

pub fn emit(items: &[Item]) -> String {
    emit_with_map(items).0
}

pub fn emit_item(item: &Item) -> String {
    let mut em = Emitter::new();
    emit_item_impl(item, &mut em, 0);
    em.text
}

fn emit_item_impl(item: &Item, em: &mut Emitter, indent: usize) {
    em.mark(item.span);
    for attr in &item.attrs {
        em.raw(&"  ".repeat(indent));
        em.raw(&format!("#[{}]\n", attr.name.text));
    }
    em.raw(&"  ".repeat(indent));
    match &item.kind {
        ItemKind::Fn(f) => emit_fn_def(f, em, indent, false),
        ItemKind::Object(o) => emit_object_def(o, em, indent),
        ItemKind::Trait(t) => emit_trait_def(t, em, indent),
        ItemKind::Error(e) => emit_error_def(e, em, indent),
        ItemKind::Enum(e) => emit_enum_def(e, em, indent),
        ItemKind::Import(i) => emit_import_def(i, em, indent),
        ItemKind::Const(c) => emit_const_def(c, em, indent),
        ItemKind::Test(t) => emit_test_def(t, em, indent),
    }
}

fn emit_fn_def(f: &FnDef, em: &mut Emitter, indent: usize, is_method: bool) {
    if !is_method {
        em.raw("fn ");
    }
    em.raw(&f.name.text);
    if !f.generics.is_empty() {
        em.raw("[");
        for (i, g) in f.generics.iter().enumerate() {
            if i > 0 { em.raw(", "); }
            em.raw(&g.text);
        }
        em.raw("]");
    }
    em.raw("(");
    for (i, p) in f.params.iter().enumerate() {
        if i > 0 { em.raw(", "); }
        em.raw(&p.name.text);
        em.raw(": ");
        emit_ty_impl(&p.ty, em);
    }
    em.raw(")");
    if let Some(ret) = &f.ret {
        em.raw(" -> ");
        emit_ty_impl(ret, em);
    }
    em.raw(" ");
    emit_block_impl(&f.body, em, indent);
}

fn emit_object_def(o: &ObjectDef, em: &mut Emitter, indent: usize) {
    em.raw("object ");
    em.raw(&o.name.text);
    if !o.generics.is_empty() {
        em.raw("[");
        for (i, g) in o.generics.iter().enumerate() {
            if i > 0 { em.raw(", "); }
            em.raw(&g.text);
        }
        em.raw("]");
    }
    em.raw(" {\n");
    for field in &o.fields {
        em.raw(&"  ".repeat(indent + 1));
        em.raw(&field.name.text);
        em.raw(": ");
        emit_ty_impl(&field.ty, em);
        em.raw(";\n");
    }
    for method in &o.methods {
        em.raw(&"  ".repeat(indent + 1));
        emit_fn_def(method, em, indent + 1, true);
        em.raw("\n");
    }
    em.raw(&"  ".repeat(indent));
    em.raw("}");
}

fn emit_trait_def(t: &TraitDef, em: &mut Emitter, indent: usize) {
    em.raw("trait ");
    em.raw(&t.name.text);
    em.raw(" {\n");
    for method in &t.methods {
        em.raw(&"  ".repeat(indent + 1));
        em.raw(&method.name.text);
        em.raw("(");
        for (i, p) in method.params.iter().enumerate() {
            if i > 0 { em.raw(", "); }
            em.raw(&p.name.text);
            em.raw(": ");
            emit_ty_impl(&p.ty, em);
        }
        em.raw(")");
        if let Some(ret) = &method.ret {
            em.raw(" -> ");
            emit_ty_impl(ret, em);
        }
        em.raw(";\n");
    }
    em.raw(&"  ".repeat(indent));
    em.raw("}");
}

fn emit_error_def(e: &ErrorDef, em: &mut Emitter, _indent: usize) {
    em.raw("error ");
    em.raw(&e.name.text);
    em.raw(" { ");
    for (i, v) in e.variants.iter().enumerate() {
        if i > 0 { em.raw(" "); }
        em.raw(&v.text);
    }
    em.raw(" }");
}

fn emit_enum_def(e: &EnumDef, em: &mut Emitter, _indent: usize) {
    em.raw("enum ");
    em.raw(&e.name.text);
    em.raw(" { ");
    for (i, v) in e.variants.iter().enumerate() {
        if i > 0 { em.raw(" "); }
        em.raw(&v.name.text);
        if !v.payload.is_empty() {
            em.raw("(");
            for (j, p) in v.payload.iter().enumerate() {
                if j > 0 { em.raw(", "); }
                emit_ty_impl(p, em);
            }
            em.raw(")");
        }
    }
    em.raw(" }");
}

fn emit_import_def(i: &ImportDef, em: &mut Emitter, _indent: usize) {
    em.raw("import ");
    for (i, p) in i.path.iter().enumerate() {
        if i > 0 { em.raw("::"); }
        em.raw(&p.text);
    }
    em.raw(";");
}

fn emit_const_def(c: &ConstDef, em: &mut Emitter, indent: usize) {
    em.raw("const ");
    em.raw(&c.name.text);
    em.raw(" = ");
    emit_expr_impl(&c.value, 0, indent, em);
    em.raw(";");
}

fn emit_test_def(t: &TestDef, em: &mut Emitter, indent: usize) {
    em.raw("test \"");
    em.raw(&t.name);
    em.raw("\" ");
    emit_block_impl(&t.body, em, indent);
}

#[allow(dead_code)]
fn emit_ty(ty: &TypeRepr) -> String {
    let mut em = Emitter::new();
    emit_ty_impl(ty, &mut em);
    em.text
}

fn emit_ty_impl(ty: &TypeRepr, em: &mut Emitter) {
    em.mark(ty.span);
    match &ty.kind {
        TypeKind::Named { name, generics } => {
            em.raw(&name.text);
            if !generics.is_empty() {
                em.raw("[");
                for (i, g) in generics.iter().enumerate() {
                    if i > 0 { em.raw(", "); }
                    emit_ty_impl(g, em);
                }
                em.raw("]");
            }
        }
        TypeKind::Optional(payload) => {
            emit_ty_impl(payload, em);
            em.raw("?");
        }
        TypeKind::Array { elem, size } => {
            emit_ty_impl(elem, em);
            em.raw("[");
            if let Some(s) = size {
                emit_expr_impl(s, 0, 0, em);
            }
            em.raw("]");
        }
        TypeKind::Fn { params, ret } => {
            em.raw("(");
            for (i, p) in params.iter().enumerate() {
                if i > 0 { em.raw(", "); }
                emit_ty_impl(p, em);
            }
            em.raw(")");
            em.raw(" -> ");
            emit_ty_impl(ret, em);
        }
        TypeKind::ErrorUnion { err, ok } => {
            emit_ty_impl(err, em);
            em.raw("!");
            emit_ty_impl(ok, em);
        }
        TypeKind::Never => {
            em.raw("!");
        }
    }
}

#[allow(dead_code)]
fn emit_block(block: &Block, indent: usize) -> String {
    let mut em = Emitter::new();
    emit_block_impl(block, &mut em, indent);
    em.text
}

fn emit_block_impl(block: &Block, em: &mut Emitter, indent: usize) {
    em.mark(block.span);
    if block.stmts.is_empty() {
        em.raw("{ }");
        return;
    }
    em.raw("{\n");
    for stmt in &block.stmts {
        em.raw(&"  ".repeat(indent + 1));
        emit_stmt_impl(stmt, em, indent + 1);
        em.raw("\n");
    }
    em.raw(&"  ".repeat(indent));
    em.raw("}");
}

#[allow(dead_code)]
fn emit_stmt(stmt: &Stmt, indent: usize) -> String {
    let mut em = Emitter::new();
    emit_stmt_impl(stmt, &mut em, indent);
    em.text
}

fn emit_stmt_impl(stmt: &Stmt, em: &mut Emitter, indent: usize) {
    em.mark(stmt.span);
    match &stmt.kind {
        StmtKind::Expr(e) => {
            emit_expr_impl(e, 0, indent, em);
            em.raw(";");
        }
        StmtKind::Assign { target, value } => {
            emit_expr_impl(target, 0, indent, em);
            em.raw(" = ");
            emit_expr_impl(value, 0, indent, em);
            em.raw(";");
        }
        StmtKind::For { var, iter, body } => {
            // "for and list comprehensions are gone after lowering (see below)"
            // But if they remain, we should print them.
            em.raw("for ");
            em.raw(&var.text);
            em.raw(" in ");
            emit_expr_impl(iter, 0, indent, em);
            em.raw(" ");
            emit_block_impl(body, em, indent);
        }
        StmtKind::Item(item) => {
            emit_item_impl(item, em, indent);
        }
        StmtKind::While { cond, body } => {
            em.raw("while (");
            emit_expr_impl(cond, 0, indent, em);
            em.raw(") ");
            emit_block_impl(body, em, indent);
        }
        StmtKind::Match { scrutinee, arms } => {
            em.raw("match (");
            emit_expr_impl(scrutinee, 0, indent, em);
            em.raw(") {\n");
            for arm in arms {
                em.raw(&"  ".repeat(indent + 1));
                emit_pattern_impl(&arm.pattern, em);
                em.raw(" => ");
                emit_stmt_impl(&arm.body, em, indent + 1);
                em.raw("\n");
            }
            em.raw(&"  ".repeat(indent));
            em.raw("}");
        }
        StmtKind::Loop { body } => {
            em.raw("loop ");
            emit_block_impl(body, em, indent);
        }
        StmtKind::Break => em.raw("break;"),
        StmtKind::Continue => em.raw("continue;"),
        StmtKind::Return(Some(e)) => {
            em.raw("return ");
            emit_expr_impl(e, 0, indent, em);
            em.raw(";");
        }
        StmtKind::Return(None) => em.raw("return;"),
    }
}

#[allow(dead_code)]
fn emit_pattern(pat: &Pattern) -> String {
    let mut em = Emitter::new();
    emit_pattern_impl(pat, &mut em);
    em.text
}

fn emit_pattern_impl(pat: &Pattern, em: &mut Emitter) {
    em.mark(pat.span);
    em.raw(&pat.name.text);
    if !pat.binds.is_empty() {
        em.raw("(");
        for (i, b) in pat.binds.iter().enumerate() {
            if i > 0 { em.raw(", "); }
            em.raw(&b.text);
        }
        em.raw(")");
    }
}

fn binop_as_str(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Rem => "%",
        BinOp::Eq => "==",
        BinOp::Ne => "!=",
        BinOp::Lt => "<",
        BinOp::Gt => ">",
        BinOp::Le => "<=",
        BinOp::Ge => ">=",
        BinOp::And => "and",
        BinOp::Or => "or",
    }
}

#[allow(dead_code)]
fn binop_precedence(op: &BinOp) -> u8 {
    match op {
        BinOp::Mul | BinOp::Div | BinOp::Rem => 7,
        BinOp::Add | BinOp::Sub => 6,
        BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => 5,
        BinOp::Eq | BinOp::Ne => 4,
        BinOp::And => 3,
        BinOp::Or => 2,
    }
}

#[allow(dead_code)]
fn emit_expr(expr: &Expr, prec: u8, indent: usize) -> String {
    let mut em = Emitter::new();
    emit_expr_impl(expr, prec, indent, &mut em);
    em.text
}


fn emit_expr_impl(expr: &Expr, prec: u8, indent: usize, em: &mut Emitter) {
    // We can just use the original emit_expr logic which returns (String, u8) and then push it to `em` and mark.
    // Wait, if we do that, the spans INSIDE the string won't be mapped!
    // We need to build it directly into `em` but track if it needs wrapping.

    // So we need a temporary emitter! Or we can just know if we need parens beforehand.
    let op_prec = match &expr.kind {
        ExprKind::Binary { op, .. } => binop_precedence(op),
        ExprKind::Lambda { .. } => 5,
        _ => 100,
    };

    let wrap = op_prec < prec;
    if wrap {
        em.raw("(");
    }

    em.mark(expr.span);

    match &expr.kind {
        ExprKind::Int(v) => em.raw(&v.to_string()),
        ExprKind::Float(v) => {
            let mut s = format!("{}", v);
            if !s.contains('.') {
                s.push_str(".0");
            }
            em.raw(&s);
        }
        ExprKind::Str(s) => {
            em.raw(&format!("\"{}\"", s.escape_debug()));
        }
        ExprKind::FStr(parts) => {
            em.raw("f\"");
            for p in parts {
                match p {
                    FPart::Text(t) => em.raw(&t.escape_debug().to_string()),
                    FPart::Expr(e) => {
                        em.raw("{");
                        em.raw(e);
                        em.raw("}");
                    }
                }
            }
            em.raw("\"");
        }
        ExprKind::Bool(v) => em.raw(if *v { "true" } else { "false" }),
        ExprKind::Null => em.raw("null"),
        ExprKind::Name(n) => em.raw(&n.text),
        ExprKind::SelfExpr => em.raw("self"),
        ExprKind::List(elems) => {
            em.raw("[");
            for (i, e) in elems.iter().enumerate() {
                if i > 0 { em.raw(", "); }
                emit_expr_impl(e, 0, indent, em);
            }
            em.raw("]");
        }
        ExprKind::Call { callee, args } => {
            emit_expr_impl(callee, 100, indent, em);
            em.raw("(");
            for (i, a) in args.iter().enumerate() {
                if i > 0 { em.raw(", "); }
                if let Some(n) = &a.name {
                    em.raw(&n.text);
                    em.raw(" = ");
                }
                emit_expr_impl(&a.value, 0, indent, em);
            }
            em.raw(")");
        }
        ExprKind::Field { base, name } => {
            emit_expr_impl(base, 100, indent, em);
            em.raw(".");
            em.raw(&name.text);
        }
        ExprKind::Index { base, index } => {
            emit_expr_impl(base, 100, indent, em);
            em.raw("[");
            emit_expr_impl(index, 0, indent, em);
            em.raw("]");
        }
        ExprKind::Unary { op, operand } => {
            em.raw("(");
            let op_str = match op {
                UnOp::Neg => "-",
                UnOp::Not => "not ",
            };
            em.raw(op_str);
            emit_expr_impl(operand, 100, indent, em);
            em.raw(")");
        }
        ExprKind::Binary { op, lhs, rhs } => {
            em.raw("(");
            emit_expr_impl(lhs, 0, indent, em);
            em.raw(&format!(" {} ", binop_as_str(op)));
            emit_expr_impl(rhs, 0, indent, em);
            em.raw(")");
        }
        ExprKind::Range { from, to } => {
            em.raw("(");
            if let Some(s) = from {
                emit_expr_impl(s, 0, indent, em);
            }
            em.raw("..");
            if let Some(e) = to {
                emit_expr_impl(e, 0, indent, em);
            }
            em.raw(")");
        }
        ExprKind::Lambda { params, body } => {
            em.raw("|");
            for (i, p) in params.iter().enumerate() {
                if i > 0 { em.raw(", "); }
                em.raw(&p.text);
            }
            em.raw("| ");
            emit_expr_impl(body, 0, indent, em);
        }
        ExprKind::Comprehension { expr, var, iter } => {
            em.raw("[");
            emit_expr_impl(expr, 0, indent, em);
            em.raw(" for ");
            em.raw(&var.text);
            em.raw(" in ");
            emit_expr_impl(iter, 0, indent, em);
            em.raw("]");
        }
        ExprKind::Propagate(e) => {
            em.raw("propagate(");
            emit_expr_impl(e, 0, indent, em);
            em.raw(")");
        }
        ExprKind::If { cond, then, elifs, else_ } => {
            em.raw("if (");
            emit_expr_impl(cond, 0, indent, em);
            em.raw(") ");
            emit_block_impl(then, em, indent);
            for (elif_cond, elif_block) in elifs {
                em.raw(" else if (");
                emit_expr_impl(elif_cond, 0, indent, em);
                em.raw(") ");
                emit_block_impl(elif_block, em, indent);
            }
            if let Some(else_block) = else_ {
                em.raw(" else ");
                emit_block_impl(else_block, em, indent);
            }
        }
    }

    if wrap {
        em.raw(")");
    }
}
