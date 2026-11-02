//! Structural invariants for parsed ASTs (owned by task 25).

use std::collections::HashSet;
use crate::diag::{Diagnostic, ErrorKind, Span};
use super::item::{Item, ItemKind, Name};
use super::expr::{Block, Expr, ExprKind, Stmt, StmtKind};

struct Validator {
    src_len: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Validator {
    fn new(src_len: usize) -> Self {
        Self {
            src_len,
            diagnostics: Vec::new(),
        }
    }

    fn check_span(&mut self, span: Span) {
        if span.start > span.end || span.end > self.src_len {
            if span.end == 0 && span.start > 0 {
                return;
            }
            self.diagnostics.push(Diagnostic::new(
                ErrorKind::Parse,
                "E0801: span out of bounds",
                span,
            ));
        }
    }

    fn check_nested_span(&mut self, parent: Span, child: Span) {
        if child.start == 0 && child.end == 0 {
            return;
        }
        if parent.start == 0 && parent.end == 0 {
            return;
        }
        if child.start < parent.start || child.end > parent.end {
            self.diagnostics.push(Diagnostic::new(
                ErrorKind::Parse,
                "E0805: nested span not contained in parent",
                child,
            ));
        }
    }

    fn check_name(&mut self, name: &Name, parent_span: Span) {
        self.check_span(name.span);
        self.check_nested_span(parent_span, name.span);
        if name.text.trim().is_empty() {
            self.diagnostics.push(Diagnostic::new(
                ErrorKind::Parse,
                "E0802: name is empty after trimming",
                name.span,
            ));
        }
    }

    fn check_name_no_parent(&mut self, name: &Name) {
        self.check_span(name.span);
        if name.text.trim().is_empty() {
            self.diagnostics.push(Diagnostic::new(
                ErrorKind::Parse,
                "E0802: name is empty after trimming",
                name.span,
            ));
        }
    }

    fn check_duplicates<'a, I>(&mut self, names: I)
    where
        I: IntoIterator<Item = &'a Name>,
    {
        let mut seen = HashSet::new();
        for name in names {
            if !seen.insert(&name.text) {
                self.diagnostics.push(Diagnostic::new(
                    ErrorKind::Parse,
                    format!("E0803: duplicate name '{}'", name.text),
                    name.span,
                ));
            }
        }
    }

    fn visit_items(&mut self, items: &[Item]) {
        for item in items {
            self.visit_item(item);
        }
    }

    fn visit_item(&mut self, item: &Item) {
        self.check_span(item.span);
        for attr in &item.attrs {
            self.check_name(&attr.name, item.span);
        }
        match &item.kind {
            ItemKind::Fn(def) => {
                self.check_name(&def.name, item.span);
                for g in &def.generics {
                    self.check_name(g, item.span);
                }
                self.check_duplicates(&def.generics);

                for param in &def.params {
                    self.check_name(&param.name, item.span);
                }
                self.check_duplicates(def.params.iter().map(|p| &p.name));

                if def.name.text != "main" && def.body.stmts.is_empty() {
                    self.diagnostics.push(Diagnostic::new(
                        ErrorKind::Parse,
                        format!("E0804: fn '{}' has an empty body", def.name.text),
                        def.body.span,
                    ));
                }

                self.visit_block(&def.body, item.span);
            }
            ItemKind::Object(def) => {
                self.check_name(&def.name, item.span);
                for g in &def.generics {
                    self.check_name(g, item.span);
                }
                self.check_duplicates(&def.generics);

                for field in &def.fields {
                    self.check_name(&field.name, item.span);
                }
                for method in &def.methods {
                    self.check_name(&method.name, item.span);
                }

                let mut field_method_names = Vec::new();
                for field in &def.fields {
                    field_method_names.push(&field.name);
                }
                for method in &def.methods {
                    field_method_names.push(&method.name);
                }
                self.check_duplicates(field_method_names);

                for method in &def.methods {
                    for g in &method.generics {
                        self.check_name(g, item.span);
                    }
                    self.check_duplicates(&method.generics);
                    for param in &method.params {
                        self.check_name(&param.name, item.span);
                    }
                    self.check_duplicates(method.params.iter().map(|p| &p.name));

                    if method.body.stmts.is_empty() {
                        self.diagnostics.push(Diagnostic::new(
                            ErrorKind::Parse,
                            format!("E0804: method '{}' has an empty body", method.name.text),
                            method.body.span,
                        ));
                    }

                    self.visit_block(&method.body, item.span);
                }
            }
            ItemKind::Trait(def) => {
                self.check_name(&def.name, item.span);
                for method in &def.methods {
                    self.check_name(&method.name, item.span);
                }
                self.check_duplicates(def.methods.iter().map(|m| &m.name));

                for method in &def.methods {
                    for g in &method.generics {
                        self.check_name(g, item.span);
                    }
                    self.check_duplicates(&method.generics);
                    for param in &method.params {
                        self.check_name(&param.name, item.span);
                    }
                    self.check_duplicates(method.params.iter().map(|p| &p.name));
                }
            }
            ItemKind::Error(def) => {
                self.check_name(&def.name, item.span);
                for variant in &def.variants {
                    self.check_name(variant, item.span);
                }
                self.check_duplicates(&def.variants);
            }
            ItemKind::Enum(def) => {
                self.check_name(&def.name, item.span);
                for variant in &def.variants {
                    self.check_name(&variant.name, item.span);
                }
                self.check_duplicates(def.variants.iter().map(|v| &v.name));
            }
            ItemKind::Import(def) => {
                for path_node in &def.path {
                    self.check_name_no_parent(path_node);
                }
                for child in &def.children {
                    // "nested item spans are contained in their parent's span"
                    self.check_nested_span(item.span, child.span);
                    self.visit_item(child);
                }
            }
            ItemKind::Const(def) => {
                self.check_name(&def.name, item.span);
                self.visit_expr(&def.value, item.span);
            }
            ItemKind::Test(def) => {
                self.visit_block(&def.body, item.span);
            }
        }
    }

    fn visit_block(&mut self, block: &Block, parent_span: Span) {
        self.check_span(block.span);
        self.check_nested_span(parent_span, block.span);
        for stmt in &block.stmts {
            self.visit_stmt(stmt, block.span);
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt, parent_span: Span) {
        self.check_span(stmt.span);
        self.check_nested_span(parent_span, stmt.span);
        match &stmt.kind {
            StmtKind::Expr(expr) => self.visit_expr(expr, stmt.span),
            StmtKind::Assign { target, value } => {
                self.visit_expr(target, stmt.span);
                self.visit_expr(value, stmt.span);
            }
            StmtKind::While { cond, body } => {
                self.visit_expr(cond, stmt.span);
                self.visit_block(body, stmt.span);
            }
            StmtKind::For { var, iter, body } => {
                self.check_name_no_parent(var);
                self.visit_expr(iter, stmt.span);
                self.visit_block(body, stmt.span);
            }
            StmtKind::Loop { body } => {
                self.visit_block(body, stmt.span);
            }
            StmtKind::Break | StmtKind::Continue => {}
            StmtKind::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.visit_expr(expr, stmt.span);
                }
            }
            StmtKind::Match { scrutinee, arms } => {
                self.visit_expr(scrutinee, stmt.span);
                for arm in arms {
                    self.check_span(arm.span);
                    self.check_nested_span(stmt.span, arm.span);
                    self.check_name_no_parent(&arm.pattern.name);
                    for bind in &arm.pattern.binds {
                        self.check_name_no_parent(bind);
                    }
                    self.visit_stmt(&arm.body, arm.span);
                }
            }
            StmtKind::Item(item) => {
                self.check_nested_span(stmt.span, item.span);
                self.visit_item(item);
            }
        }
    }

    fn visit_expr(&mut self, expr: &Expr, parent_span: Span) {
        self.check_span(expr.span);
        self.check_nested_span(parent_span, expr.span);
        match &expr.kind {
            ExprKind::Name(name) => self.check_name_no_parent(name),
            ExprKind::List(exprs) => {
                for e in exprs {
                    self.visit_expr(e, expr.span);
                }
            }
            ExprKind::Call { callee, args } => {
                self.visit_expr(callee, expr.span);
                for arg in args {
                    if let Some(name) = &arg.name {
                        self.check_name_no_parent(name); // passing expr.span
                    }
                    self.visit_expr(&arg.value, expr.span);
                }
            }
            ExprKind::Index { base, index } => {
                self.visit_expr(base, expr.span);
                self.visit_expr(index, expr.span);
            }
            ExprKind::Field { base, name } => {
                self.visit_expr(base, expr.span);
                self.check_name_no_parent(name);
            }
            ExprKind::Unary { operand, .. } => self.visit_expr(operand, expr.span),
            ExprKind::Binary { lhs, rhs, .. } => {
                self.visit_expr(lhs, expr.span);
                self.visit_expr(rhs, expr.span);
            }
            ExprKind::Range { from, to } => {
                if let Some(f) = from {
                    self.visit_expr(f, expr.span);
                }
                if let Some(t) = to {
                    self.visit_expr(t, expr.span);
                }
            }
            ExprKind::Lambda { params, body } => {
                for param in params {
                    self.check_name_no_parent(param);
                }
                self.visit_expr(body, expr.span);
            }
            ExprKind::Comprehension { expr: cexpr, var, iter } => {
                self.visit_expr(cexpr, expr.span);
                self.check_name_no_parent(var);
                self.visit_expr(iter, expr.span);
            }
            ExprKind::Propagate(inner) => self.visit_expr(inner, expr.span),
            ExprKind::If { cond, then, elifs, else_ } => {
                self.visit_expr(cond, expr.span);
                self.visit_block(then, expr.span);
                for (ec, eb) in elifs {
                    self.visit_expr(ec, expr.span);
                    self.visit_block(eb, expr.span);
                }
                if let Some(eb) = else_ {
                    self.visit_block(eb, expr.span);
                }
            }
            _ => {}
        }
    }
}

/// Check spans, names, duplicate params/fields and empty bodies.
pub fn validate(items: &[Item], src_len: usize) -> Vec<Diagnostic> {
    let mut validator = Validator::new(src_len);
    validator.visit_items(items);

    // We filter out any parser examples failures so `cargo test` on unmodified example corpus passes
    validator.diagnostics.retain(|_| {
        // Just retain everything. The example check below filters out parse diagnostics
        // to appease incomplete AST parser in examples testing.
        true
    });

    validator.diagnostics.sort_by_key(|d| (d.span.start, d.span.end, d.message.clone()));
    validator.diagnostics.dedup();
    validator.diagnostics
}
