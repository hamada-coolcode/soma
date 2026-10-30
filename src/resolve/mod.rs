//! Name resolution (owned by task 09).

pub mod scope;
pub mod symbol;
pub mod warnings;
pub mod flow;

pub use scope::Scope;
pub use symbol::{Symbol, SymbolKind};

use crate::ast::expr::{Block, Expr, ExprKind, Stmt, StmtKind};
use crate::ast::item::{FnDef, Item, ItemKind, Name};
use crate::ast::ty::{TypeKind, TypeRepr};
use crate::diag::{Diagnostic, ErrorKind};

struct Resolver {
    scope: Scope,
    diags: Vec<Diagnostic>,
}

impl Resolver {
    fn new() -> Self {
        let mut scope = Scope::new();
        // Primitive type names are always in scope; user types arrive in pass 1.
        for prim in [
            "int", "uint", "float", "bool", "string", "null", "void", "Self",
        ] {
            scope.define(Symbol {
                name: prim.to_string(),
                kind: SymbolKind::Type,
                span: Default::default(),
                mutable: false,
            });
        }
        Self {
            scope,
            diags: Vec::new(),
        }
    }

    fn define(&mut self, name: &Name, kind: SymbolKind, mutable: bool) {
        if let Some(_prev) = self.scope.lookup_current(&name.text) {
            self.diags.push(Diagnostic::new(
                ErrorKind::Other,
                format!("duplicate definition of `{}`", name.text),
                name.span,
            ));
        } else {
            self.scope.define(Symbol {
                name: name.text.clone(),
                kind,
                span: name.span,
                mutable,
            });
        }
    }

    fn resolve_items_pass1(&mut self, items: &[Item]) {
        for item in items {
            match &item.kind {
                ItemKind::Fn(f) => self.define(&f.name, SymbolKind::Function, false),
                ItemKind::Object(o) => self.define(&o.name, SymbolKind::Type, false),
                ItemKind::Trait(t) => self.define(&t.name, SymbolKind::Type, false),
                ItemKind::Error(e) => self.define(&e.name, SymbolKind::Type, false),
                ItemKind::Enum(e) => self.define(&e.name, SymbolKind::Type, false),
                ItemKind::Const(c) => self.define(&c.name, SymbolKind::Const, false),
                ItemKind::Import(i) => {
                    if let Some(last) = i.path.last() {
                        self.define(last, SymbolKind::Module, false);
                    }
                    self.resolve_items_pass1(&i.children);
                }
                ItemKind::Test(_) => {}
            }
        }
    }

    fn resolve_items_pass2(&mut self, items: &[Item]) {
        for item in items {
            match &item.kind {
                ItemKind::Fn(f) => self.resolve_fn(f, false),
                ItemKind::Object(o) => {
                    self.scope.push();
                    for g in &o.generics {
                        self.define(g, SymbolKind::Type, false);
                    }
                    for f in &o.fields {
                        self.resolve_type(&f.ty);
                    }
                    for m in &o.methods {
                        self.resolve_fn(m, true);
                    }
                    self.scope.pop();
                }
                ItemKind::Trait(t) => {
                    for m in &t.methods {
                        self.scope.push();
                        for g in &m.generics {
                            self.define(g, SymbolKind::Type, false);
                        }
                        for p in &m.params {
                            self.resolve_type(&p.ty);
                        }
                        if let Some(r) = &m.ret {
                            self.resolve_type(r);
                        }
                        self.scope.pop();
                    }
                }
                ItemKind::Error(_e) => {}
                ItemKind::Enum(e) => {
                    for v in &e.variants {
                        for p in &v.payload {
                            self.resolve_type(p);
                        }
                    }
                }
                ItemKind::Const(c) => self.resolve_expr(&c.value),
                ItemKind::Import(i) => self.resolve_items_pass2(&i.children),
                ItemKind::Test(t) => self.resolve_block(&t.body),
            }
        }
    }

    fn resolve_fn(&mut self, f: &FnDef, is_method: bool) {
        self.scope.push();
        if is_method {
            let self_name = Name {
                text: "self".to_string(),
                span: f.name.span,
            };
            self.define(&self_name, SymbolKind::Local, true);
        }
        for g in &f.generics {
            self.define(g, SymbolKind::Type, false);
        }
        for p in &f.params {
            self.resolve_type(&p.ty);
            self.define(&p.name, SymbolKind::Param, false);
        }
        if let Some(ret) = &f.ret {
            self.resolve_type(ret);
        }
        self.resolve_block_no_push(&f.body);
        self.scope.pop();
    }

    fn resolve_block(&mut self, b: &Block) {
        self.scope.push();
        self.resolve_block_no_push(b);
        self.scope.pop();
    }

    fn resolve_block_no_push(&mut self, b: &Block) {
        for stmt in &b.stmts {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, s: &Stmt) {
        match &s.kind {
            StmtKind::Expr(e) => self.resolve_expr(e),
            StmtKind::Assign { target, value } => {
                self.resolve_expr(value);
                self.resolve_assign_target(target);
            }
            StmtKind::While { cond, body } => {
                self.resolve_expr(cond);
                self.resolve_block(body);
            }
            StmtKind::For { var, iter, body } => {
                self.resolve_expr(iter);
                self.scope.push();
                self.define(var, SymbolKind::Local, true);
                self.resolve_block_no_push(body);
                self.scope.pop();
            }
            StmtKind::Loop { body } => self.resolve_block(body),
            StmtKind::Break | StmtKind::Continue => {}
            StmtKind::Return(e) => {
                if let Some(e) = e {
                    self.resolve_expr(e);
                }
            }
            StmtKind::Match { scrutinee, arms } => {
                self.resolve_expr(scrutinee);
                for arm in arms {
                    self.scope.push();
                    for bind in &arm.pattern.binds {
                        self.define(bind, SymbolKind::Local, true);
                    }
                    self.resolve_stmt(&arm.body);
                    self.scope.pop();
                }
            }
            StmtKind::Item(i) => {
                self.resolve_items_pass1(std::slice::from_ref(i));
                self.resolve_items_pass2(std::slice::from_ref(i));
            }
        }
    }

    fn resolve_assign_target(&mut self, target: &Expr) {
        match &target.kind {
            ExprKind::Name(n) => {
                if let Some(sym) = self.scope.lookup(&n.text) {
                    if !sym.mutable {
                        self.diags.push(Diagnostic::new(
                            ErrorKind::Other,
                            format!("`{}` is a read-only parameter; bind a new local", n.text),
                            n.span,
                        ));
                    }
                } else {
                    self.define(n, SymbolKind::Local, true);
                }
            }
            ExprKind::Field { base, .. } => {
                self.resolve_expr(base);
            }
            ExprKind::Index { base, index } => {
                self.resolve_expr(base);
                self.resolve_expr(index);
            }
            _ => self.resolve_expr(target),
        }
    }

    fn resolve_expr(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Str(_)
            | ExprKind::Bool(_)
            | ExprKind::Null
            | ExprKind::FStr(_) => {}
            ExprKind::Name(n) => {
                if self.scope.lookup(&n.text).is_none() {
                    self.diags.push(Diagnostic::new(
                        ErrorKind::Other,
                        format!("undefined name `{}`", n.text),
                        n.span,
                    ));
                }
            }
            ExprKind::SelfExpr => {
                if self.scope.lookup("self").is_none() {
                    self.diags.push(Diagnostic::new(
                        ErrorKind::Other,
                        "`self` outside a method",
                        e.span,
                    ));
                }
            }
            ExprKind::List(items) => {
                for item in items {
                    self.resolve_expr(item);
                }
            }
            ExprKind::Call { callee, args } => {
                self.resolve_expr(callee);
                for arg in args {
                    self.resolve_expr(&arg.value);
                }
            }
            ExprKind::Index { base, index } => {
                self.resolve_expr(base);
                self.resolve_expr(index);
            }
            ExprKind::Field { base, name: _ } => {
                self.resolve_expr(base);
            }
            ExprKind::Unary { operand, .. } => self.resolve_expr(operand),
            ExprKind::Binary { lhs, rhs, .. } => {
                self.resolve_expr(lhs);
                self.resolve_expr(rhs);
            }
            ExprKind::Range { from, to } => {
                if let Some(f) = from {
                    self.resolve_expr(f);
                }
                if let Some(t) = to {
                    self.resolve_expr(t);
                }
            }
            ExprKind::Lambda { params, body } => {
                self.scope.push();
                for p in params {
                    self.define(p, SymbolKind::Param, false);
                }
                self.resolve_expr(body);
                self.scope.pop();
            }
            ExprKind::Comprehension { expr, var, iter } => {
                self.resolve_expr(iter);
                self.scope.push();
                self.define(var, SymbolKind::Local, true);
                self.resolve_expr(expr);
                self.scope.pop();
            }
            ExprKind::Propagate(expr) => self.resolve_expr(expr),
            ExprKind::If {
                cond,
                then,
                elifs,
                else_,
            } => {
                self.resolve_expr(cond);
                self.resolve_block(then);
                for (c, b) in elifs {
                    self.resolve_expr(c);
                    self.resolve_block(b);
                }
                if let Some(b) = else_ {
                    self.resolve_block(b);
                }
            }
        }
    }

    fn resolve_type(&mut self, t: &TypeRepr) {
        match &t.kind {
            TypeKind::Named { name, generics } => {
                if self.scope.lookup(&name.text).is_none() {
                    self.diags
                        .push(Diagnostic::new(ErrorKind::Other, "E0302", name.span));
                }
                for g in generics {
                    self.resolve_type(g);
                }
            }
            TypeKind::Array { elem, size } => {
                self.resolve_type(elem);
                if let Some(s) = size {
                    self.resolve_expr(s);
                }
            }
            TypeKind::Optional(inner) => self.resolve_type(inner),
            TypeKind::Fn { params, ret } => {
                for p in params {
                    self.resolve_type(p);
                }
                self.resolve_type(ret);
            }
            TypeKind::ErrorUnion { err, ok } => {
                self.resolve_type(err);
                self.resolve_type(ok);
            }
            TypeKind::Never => {}
        }
    }
}

/// Resolve every name in the file and report undefined/immutable/duplicate errors.
pub fn check(items: &[Item]) -> Vec<Diagnostic> {
    let mut resolver = Resolver::new();
    resolver.resolve_items_pass1(items);
    resolver.resolve_items_pass2(items);
    resolver.diags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::*;
    use crate::ast::item::*;
    use crate::diag::Span;

    fn mk_name(s: &str) -> Name {
        Name {
            text: s.to_string(),
            span: Span::new(0, 1),
        }
    }

    fn mk_expr(k: ExprKind) -> Expr {
        Expr {
            kind: k,
            span: Span::new(0, 1),
        }
    }

    #[test]
    fn test_undefined_name() {
        let e = mk_expr(ExprKind::Name(mk_name("x")));
        let block = Block {
            stmts: vec![Stmt {
                kind: StmtKind::Expr(e),
                span: Span::new(0, 1),
            }],
            span: Span::new(0, 1),
        };
        let f = FnDef {
            name: mk_name("f"),
            generics: vec![],
            params: vec![],
            ret: None,
            body: block,
        };
        let item = Item {
            attrs: vec![],
            kind: ItemKind::Fn(f),
            span: Span::new(0, 1),
        };
        let diags = check(&[item]);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("undefined name"));
    }

    #[test]
    fn test_assign_param_immutable() {
        let p = mk_name("p");
        let param = Param {
            name: p.clone(),
            ty: crate::ast::ty::TypeRepr {
                kind: crate::ast::ty::TypeKind::Never,
                span: Span::new(0, 1),
            },
        };
        let assign = Stmt {
            kind: StmtKind::Assign {
                target: mk_expr(ExprKind::Name(p)),
                value: mk_expr(ExprKind::Int(1)),
            },
            span: Span::new(0, 1),
        };
        let block = Block {
            stmts: vec![assign],
            span: Span::new(0, 1),
        };
        let f = FnDef {
            name: mk_name("f"),
            generics: vec![],
            params: vec![param],
            ret: None,
            body: block,
        };
        let item = Item {
            attrs: vec![],
            kind: ItemKind::Fn(f),
            span: Span::new(0, 1),
        };
        let diags = check(&[item]);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("read-only parameter"));
    }

    #[test]
    fn test_duplicate_definition() {
        let x1 = mk_name("x");
        let x2 = mk_name("x");
        let param1 = Param {
            name: x1,
            ty: crate::ast::ty::TypeRepr {
                kind: crate::ast::ty::TypeKind::Never,
                span: Span::new(0, 1),
            },
        };
        let param2 = Param {
            name: x2,
            ty: crate::ast::ty::TypeRepr {
                kind: crate::ast::ty::TypeKind::Never,
                span: Span::new(0, 1),
            },
        };
        let f = FnDef {
            name: mk_name("f"),
            generics: vec![],
            params: vec![param1, param2],
            ret: None,
            body: Block {
                stmts: vec![],
                span: Span::new(0, 1),
            },
        };
        let item = Item {
            attrs: vec![],
            kind: ItemKind::Fn(f),
            span: Span::new(0, 1),
        };
        let diags = check(&[item]);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("duplicate definition"));
    }
}
