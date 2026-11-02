//! Machine-readable AST dump (owned by task 22).

use crate::ast::expr::{Arm, BinOp, Block, Expr, ExprKind, Pattern, Stmt, StmtKind, UnOp};
use crate::ast::item::{Item, ItemKind, Name};
use crate::ast::ty::{TypeKind, TypeRepr};
use crate::diag::Span;
use crate::lexer::token::FPart;

pub fn emit_json(items: &[Item]) -> String {
    let mut printer = JsonPrinter::new();
    printer.array(|p| {
        for item in items {
            p.array_entry(|p| p.item(item));
        }
    });
    let mut out = printer.out;
    out.push('\n');
    out
}

struct JsonPrinter {
    out: String,
    indent: usize,
    comma_needed: Vec<bool>,
}

impl JsonPrinter {
    fn new() -> Self {
        Self {
            out: String::new(),
            indent: 0,
            comma_needed: vec![false],
        }
    }

    fn push_indent(&mut self) {
        self.out.push_str(&"  ".repeat(self.indent));
    }

    fn prepare_entry(&mut self) {
        if *self.comma_needed.last().unwrap() {
            self.out.push(',');
            self.out.push('\n');
        } else {
            *self.comma_needed.last_mut().unwrap() = true;
            if self.out.ends_with(|c| c == '{' || c == '[') {
                self.out.push('\n');
            }
        }
        self.push_indent();
    }

    fn array_entry<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.prepare_entry();
        let was = *self.comma_needed.last().unwrap();
        self.comma_needed.push(false);
        f(self);
        self.comma_needed.pop();
        *self.comma_needed.last_mut().unwrap() = was;
    }

    fn escape(s: &str) -> String {
        let mut out = String::new();
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\t' => out.push_str("\\t"),
                '\r' => out.push_str("\\r"),
                c if c < '\x20' => out.push_str(&format!("\\u{:04x}", c as u32)),
                _ => out.push(c),
            }
        }
        out
    }

    fn object<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.out.push('{');
        self.indent += 1;
        self.comma_needed.push(false);
        f(self);
        let has_content = *self.comma_needed.last().unwrap();
        self.indent -= 1;
        self.comma_needed.pop();
        if has_content {
            self.out.push('\n');
            self.push_indent();
        }
        self.out.push('}');
    }

    fn array<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.out.push('[');
        self.indent += 1;
        self.comma_needed.push(false);
        f(self);
        let has_content = *self.comma_needed.last().unwrap();
        self.indent -= 1;
        self.comma_needed.pop();
        if has_content {
            self.out.push('\n');
            self.push_indent();
        }
        self.out.push(']');
    }

    fn key(&mut self, key: &str) {
        self.prepare_entry();
        self.out.push('"');
        self.out.push_str(key);
        self.out.push_str("\": ");
        self.comma_needed.push(false);
    }

    fn end_val(&mut self) {
        self.comma_needed.pop();
        *self.comma_needed.last_mut().unwrap() = true;
    }

    fn value_str(&mut self, s: &str) {
        self.out.push('"');
        self.out.push_str(&Self::escape(s));
        self.out.push('"');
        self.end_val();
    }

    fn value_bool(&mut self, b: bool) {
        self.out.push_str(if b { "true" } else { "false" });
        self.end_val();
    }

    fn value_int(&mut self, i: i64) {
        self.out.push_str(&i.to_string());
        self.end_val();
    }

    fn value_float(&mut self, f: f64) {
        self.out.push_str(&f.to_string());
        self.end_val();
    }

    fn value_null(&mut self) {
        self.out.push_str("null");
        self.end_val();
    }

    fn span(&mut self, span: &Span) {
        self.key("span");
        self.object(|p| {
            p.key("start");
            p.value_int(span.start as i64);
            p.key("end");
            p.value_int(span.end as i64);
        });
        self.end_val();
    }

    fn name(&mut self, name: &Name) {
        self.object(|p| {
            p.key("text");
            p.value_str(&name.text);
            p.span(&name.span);
        });
    }

    fn type_repr(&mut self, ty: &TypeRepr) {
        self.object(|p| {
            p.key("kind");
            match &ty.kind {
                TypeKind::Named { name, generics } => {
                    p.value_str("Named");
                    p.key("name");
                    p.name(name);
                    p.key("generics");
                    p.array(|p| {
                        for g in generics {
                            p.array_entry(|p| p.type_repr(g));
                        }
                    });
                    p.end_val();
                }
                TypeKind::Array { elem, size } => {
                    p.value_str("Array");
                    p.key("elem");
                    p.type_repr(elem);
                    p.key("size");
                    if let Some(s) = size {
                        p.expr(s);
                    } else {
                        p.value_null();
                    }
                }
                TypeKind::Optional(inner) => {
                    p.value_str("Optional");
                    p.key("inner");
                    p.type_repr(inner);
                }
                TypeKind::Fn { params, ret } => {
                    p.value_str("Fn");
                    p.key("params");
                    p.array(|p| {
                        for param in params {
                            p.array_entry(|p| p.type_repr(param));
                        }
                    });
                    p.end_val();
                    p.key("ret");
                    p.type_repr(ret);
                }
                TypeKind::ErrorUnion { err, ok } => {
                    p.value_str("ErrorUnion");
                    p.key("err");
                    p.type_repr(err);
                    p.key("ok");
                    p.type_repr(ok);
                }
                TypeKind::Never => {
                    p.value_str("Never");
                }
            }
            p.span(&ty.span);
        });
    }

    fn expr(&mut self, expr: &Expr) {
        self.object(|p| {
            p.key("kind");
            match &expr.kind {
                ExprKind::Int(v) => {
                    p.value_str("Int");
                    p.key("value");
                    p.value_int(*v);
                }
                ExprKind::Float(v) => {
                    p.value_str("Float");
                    p.key("value");
                    p.value_float(*v);
                }
                ExprKind::Str(v) => {
                    p.value_str("Str");
                    p.key("value");
                    p.value_str(v);
                }
                ExprKind::FStr(parts) => {
                    p.value_str("FStr");
                    p.key("parts");
                    p.array(|p| {
                        for part in parts {
                            p.array_entry(|p| {
                                p.object(|p| {
                                    p.key("kind");
                                    match part {
                                        FPart::Text(t) => {
                                            p.value_str("Text");
                                            p.key("value");
                                            p.value_str(t);
                                        }
                                        FPart::Expr(e) => {
                                            p.value_str("Expr");
                                            p.key("value");
                                            p.value_str(e);
                                        }
                                    }
                                });
                            });
                        }
                    });
                    p.end_val();
                }
                ExprKind::Bool(v) => {
                    p.value_str("Bool");
                    p.key("value");
                    p.value_bool(*v);
                }
                ExprKind::Null => {
                    p.value_str("Null");
                }
                ExprKind::Name(n) => {
                    p.value_str("Name");
                    p.key("name");
                    p.name(n);
                }
                ExprKind::SelfExpr => {
                    p.value_str("SelfExpr");
                }
                ExprKind::List(exprs) => {
                    p.value_str("List");
                    p.key("exprs");
                    p.array(|p| {
                        for e in exprs {
                            p.array_entry(|p| p.expr(e));
                        }
                    });
                    p.end_val();
                }
                ExprKind::Call { callee, args } => {
                    p.value_str("Call");
                    p.key("callee");
                    p.expr(callee);
                    p.key("args");
                    p.array(|p| {
                        for arg in args {
                            p.array_entry(|p| {
                                p.object(|p| {
                                    p.key("name");
                                    if let Some(n) = &arg.name {
                                        p.name(n);
                                    } else {
                                        p.value_null();
                                    }
                                    p.key("value");
                                    p.expr(&arg.value);
                                });
                            });
                        }
                    });
                    p.end_val();
                }
                ExprKind::Index { base, index } => {
                    p.value_str("Index");
                    p.key("base");
                    p.expr(base);
                    p.key("index");
                    p.expr(index);
                }
                ExprKind::Field { base, name } => {
                    p.value_str("Field");
                    p.key("base");
                    p.expr(base);
                    p.key("name");
                    p.name(name);
                }
                ExprKind::Unary { op, operand } => {
                    p.value_str("Unary");
                    p.key("op");
                    p.value_str(match op {
                        UnOp::Not => "Not",
                        UnOp::Neg => "Neg",
                    });
                    p.key("operand");
                    p.expr(operand);
                }
                ExprKind::Binary { op, lhs, rhs } => {
                    p.value_str("Binary");
                    p.key("op");
                    p.value_str(match op {
                        BinOp::Add => "Add",
                        BinOp::Sub => "Sub",
                        BinOp::Mul => "Mul",
                        BinOp::Div => "Div",
                        BinOp::Rem => "Rem",
                        BinOp::Eq => "Eq",
                        BinOp::Ne => "Ne",
                        BinOp::Lt => "Lt",
                        BinOp::Gt => "Gt",
                        BinOp::Le => "Le",
                        BinOp::Ge => "Ge",
                        BinOp::And => "And",
                        BinOp::Or => "Or",
                    });
                    p.key("lhs");
                    p.expr(lhs);
                    p.key("rhs");
                    p.expr(rhs);
                }
                ExprKind::Range { from, to } => {
                    p.value_str("Range");
                    p.key("from");
                    if let Some(f) = from {
                        p.expr(f);
                    } else {
                        p.value_null();
                    }
                    p.key("to");
                    if let Some(t) = to {
                        p.expr(t);
                    } else {
                        p.value_null();
                    }
                }
                ExprKind::Lambda { params, body } => {
                    p.value_str("Lambda");
                    p.key("params");
                    p.array(|p| {
                        for param in params {
                            p.array_entry(|p| p.name(param));
                        }
                    });
                    p.end_val();
                    p.key("body");
                    p.expr(body);
                }
                ExprKind::Comprehension { expr, var, iter } => {
                    p.value_str("Comprehension");
                    p.key("expr");
                    p.expr(expr);
                    p.key("var");
                    p.name(var);
                    p.key("iter");
                    p.expr(iter);
                }
                ExprKind::Propagate(expr) => {
                    p.value_str("Propagate");
                    p.key("expr");
                    p.expr(expr);
                }
                ExprKind::If {
                    cond,
                    then,
                    elifs,
                    else_,
                } => {
                    p.value_str("If");
                    p.key("cond");
                    p.expr(cond);
                    p.key("then");
                    p.block(then);
                    p.key("elifs");
                    p.array(|p| {
                        for (c, b) in elifs {
                            p.array_entry(|p| {
                                p.object(|p| {
                                    p.key("cond");
                                    p.expr(c);
                                    p.key("body");
                                    p.block(b);
                                });
                            });
                        }
                    });
                    p.end_val();
                    p.key("else");
                    if let Some(e) = else_ {
                        p.block(e);
                    } else {
                        p.value_null();
                    }
                }
            }
            p.span(&expr.span);
        });
    }

    fn stmt(&mut self, stmt: &Stmt) {
        self.object(|p| {
            p.key("kind");
            match &stmt.kind {
                StmtKind::Expr(expr) => {
                    p.value_str("Expr");
                    p.key("expr");
                    p.expr(expr);
                }
                StmtKind::Assign { target, value } => {
                    p.value_str("Assign");
                    p.key("target");
                    p.expr(target);
                    p.key("value");
                    p.expr(value);
                }
                StmtKind::While { cond, body } => {
                    p.value_str("While");
                    p.key("cond");
                    p.expr(cond);
                    p.key("body");
                    p.block(body);
                }
                StmtKind::For { var, iter, body } => {
                    p.value_str("For");
                    p.key("var");
                    p.name(var);
                    p.key("iter");
                    p.expr(iter);
                    p.key("body");
                    p.block(body);
                }
                StmtKind::Loop { body } => {
                    p.value_str("Loop");
                    p.key("body");
                    p.block(body);
                }
                StmtKind::Break => {
                    p.value_str("Break");
                }
                StmtKind::Continue => {
                    p.value_str("Continue");
                }
                StmtKind::Return(expr) => {
                    p.value_str("Return");
                    p.key("expr");
                    if let Some(e) = expr {
                        p.expr(e);
                    } else {
                        p.value_null();
                    }
                }
                StmtKind::Match { scrutinee, arms } => {
                    p.value_str("Match");
                    p.key("scrutinee");
                    p.expr(scrutinee);
                    p.key("arms");
                    p.array(|p| {
                        for arm in arms {
                            p.array_entry(|p| p.arm(arm));
                        }
                    });
                    p.end_val();
                }
                StmtKind::Item(item) => {
                    p.value_str("Item");
                    p.key("item");
                    p.item(item);
                }
            }
            p.span(&stmt.span);
        });
    }

    fn block(&mut self, block: &Block) {
        self.object(|p| {
            p.key("stmts");
            p.array(|p| {
                for stmt in &block.stmts {
                    p.array_entry(|p| p.stmt(stmt));
                }
            });
            p.end_val();
            p.span(&block.span);
        });
    }

    fn arm(&mut self, arm: &Arm) {
        self.object(|p| {
            p.key("pattern");
            p.pattern(&arm.pattern);
            p.key("body");
            p.stmt(&arm.body);
            p.span(&arm.span);
        });
    }

    fn pattern(&mut self, pat: &Pattern) {
        self.object(|p| {
            p.key("name");
            p.name(&pat.name);
            p.key("binds");
            p.array(|p| {
                for bind in &pat.binds {
                    p.array_entry(|p| p.name(bind));
                }
            });
            p.end_val();
            p.span(&pat.span);
        });
    }

    fn item(&mut self, item: &Item) {
        self.object(|p| {
            p.key("attrs");
            p.array(|p| {
                for attr in &item.attrs {
                    p.array_entry(|p| {
                        p.object(|p| {
                            p.key("name");
                            p.name(&attr.name);
                        });
                    });
                }
            });
            p.end_val();
            p.key("kind");
            match &item.kind {
                ItemKind::Fn(f) => {
                    p.value_str("Fn");
                    p.key("name");
                    p.name(&f.name);
                    p.key("generics");
                    p.array(|p| {
                        for g in &f.generics {
                            p.array_entry(|p| p.name(g));
                        }
                    });
                    p.end_val();
                    p.key("params");
                    p.array(|p| {
                        for param in &f.params {
                            p.array_entry(|p| {
                                p.object(|p| {
                                    p.key("name");
                                    p.name(&param.name);
                                    p.key("ty");
                                    p.type_repr(&param.ty);
                                });
                            });
                        }
                    });
                    p.end_val();
                    p.key("ret");
                    if let Some(r) = &f.ret {
                        p.type_repr(r);
                    } else {
                        p.value_null();
                    }
                    p.key("body");
                    p.block(&f.body);
                }
                ItemKind::Object(o) => {
                    p.value_str("Object");
                    p.key("name");
                    p.name(&o.name);
                    p.key("generics");
                    p.array(|p| {
                        for g in &o.generics {
                            p.array_entry(|p| p.name(g));
                        }
                    });
                    p.end_val();
                    p.key("fields");
                    p.array(|p| {
                        for field in &o.fields {
                            p.array_entry(|p| {
                                p.object(|p| {
                                    p.key("name");
                                    p.name(&field.name);
                                    p.key("ty");
                                    p.type_repr(&field.ty);
                                });
                            });
                        }
                    });
                    p.end_val();
                    p.key("methods");
                    p.array(|p| {
                        for method in &o.methods {
                            p.array_entry(|p| {
                                p.item(&Item {
                                    attrs: vec![],
                                    kind: ItemKind::Fn(method.clone()),
                                    span: Span::new(0, 0), // Dummy span for inner method
                                })
                            });
                        }
                    });
                    p.end_val();
                }
                ItemKind::Trait(t) => {
                    p.value_str("Trait");
                    p.key("name");
                    p.name(&t.name);
                    p.key("methods");
                    p.array(|p| {
                        for method in &t.methods {
                            p.array_entry(|p| {
                                p.object(|p| {
                                    p.key("name");
                                    p.name(&method.name);
                                    p.key("generics");
                                    p.array(|p| {
                                        for g in &method.generics {
                                            p.array_entry(|p| p.name(g));
                                        }
                                    });
                                    p.end_val();
                                    p.key("params");
                                    p.array(|p| {
                                        for param in &method.params {
                                            p.array_entry(|p| {
                                                p.object(|p| {
                                                    p.key("name");
                                                    p.name(&param.name);
                                                    p.key("ty");
                                                    p.type_repr(&param.ty);
                                                });
                                            });
                                        }
                                    });
                                    p.end_val();
                                    p.key("ret");
                                    if let Some(r) = &method.ret {
                                        p.type_repr(r);
                                    } else {
                                        p.value_null();
                                    }
                                });
                            });
                        }
                    });
                    p.end_val();
                }
                ItemKind::Error(e) => {
                    p.value_str("Error");
                    p.key("name");
                    p.name(&e.name);
                    p.key("variants");
                    p.array(|p| {
                        for v in &e.variants {
                            p.array_entry(|p| p.name(v));
                        }
                    });
                    p.end_val();
                }
                ItemKind::Enum(e) => {
                    p.value_str("Enum");
                    p.key("name");
                    p.name(&e.name);
                    p.key("variants");
                    p.array(|p| {
                        for v in &e.variants {
                            p.array_entry(|p| {
                                p.object(|p| {
                                    p.key("name");
                                    p.name(&v.name);
                                    p.key("payload");
                                    p.array(|p| {
                                        for p_type in &v.payload {
                                            p.array_entry(|p| p.type_repr(p_type));
                                        }
                                    });
                                    p.end_val();
                                });
                            });
                        }
                    });
                    p.end_val();
                }
                ItemKind::Import(i) => {
                    p.value_str("Import");
                    p.key("path");
                    p.array(|p| {
                        for n in &i.path {
                            p.array_entry(|p| p.name(n));
                        }
                    });
                    p.end_val();
                    p.key("children");
                    p.array(|p| {
                        for c in &i.children {
                            p.array_entry(|p| p.item(c));
                        }
                    });
                    p.end_val();
                }
                ItemKind::Const(c) => {
                    p.value_str("Const");
                    p.key("name");
                    p.name(&c.name);
                    p.key("value");
                    p.expr(&c.value);
                }
                ItemKind::Test(t) => {
                    p.value_str("Test");
                    p.key("name");
                    p.value_str(&t.name);
                    p.key("body");
                    p.block(&t.body);
                }
            }
            p.span(&item.span);
        });
    }
}
