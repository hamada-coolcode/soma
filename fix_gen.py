import sys
with open("src/types/check_item.rs", "w") as f:
    f.write("""use std::collections::{HashMap, HashSet};

use crate::ast::item::{ConstDef, FnDef, Item, ItemKind, ObjectDef};
use crate::ast::expr::{Expr, ExprKind, StmtKind, Block};
use crate::diag::{Diagnostic, ErrorKind, Span};
use crate::types::{check_expr, unify, Type, TypeId, TypeTable};

/// Check top-level items (owned by task 12).
pub fn check_items(items: &[Item]) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let mut table = TypeTable::new();
    let mut env = HashMap::new();

    for item in items {
        match &item.kind {
            ItemKind::Fn(f) => {
                check_fn(&mut table, &mut env, f, &mut diags);
            }
            ItemKind::Object(obj) => {
                check_object(&mut table, &mut env, obj, &mut diags);
            }
            ItemKind::Const(c) => {
                check_const(&mut table, &mut env, c, &mut diags);
            }
            _ => {}
        }
    }

    diags
}

fn check_fn(
    table: &mut TypeTable,
    env: &mut HashMap<String, u32>,
    f: &FnDef,
    diags: &mut Vec<Diagnostic>,
) {
    let mut generic_names = HashSet::new();
    for g in &f.generics {
        if !generic_names.insert(&g.text) {
            diags.push(Diagnostic::new(
                ErrorKind::Type,
                format!("duplicate generic parameter `{}`", g.text),
                g.span.clone(),
            ));
        } else {
            let var = table.fresh_var();
            if let Type::Var(id) = table.resolve(var) {
                env.insert(g.text.clone(), id);
            }
        }
    }

    let mut param_names = HashSet::new();
    for p in &f.params {
        if !param_names.insert(&p.name.text) {
            diags.push(Diagnostic::new(
                ErrorKind::Type,
                format!("duplicate parameter name `{}`", p.name.text),
                p.name.span.clone(),
            ));
        }
        let _ty_id = table.from_repr(&p.ty, env);
    }

    let ret_id = if let Some(ret) = &f.ret {
        table.from_repr(ret, env)
    } else {
        table.intern(Type::Unit)
    };

    check_block(table, env, &f.body, ret_id, f.name.text == "main", diags);

    for g in &f.generics {
        env.remove(&g.text);
    }
}

fn check_object(
    table: &mut TypeTable,
    env: &mut HashMap<String, u32>,
    obj: &ObjectDef,
    diags: &mut Vec<Diagnostic>,
) {
    let mut generic_names = HashSet::new();
    for g in &obj.generics {
        if !generic_names.insert(&g.text) {
            diags.push(Diagnostic::new(
                ErrorKind::Type,
                format!("duplicate generic parameter `{}`", g.text),
                g.span.clone(),
            ));
        } else {
            let var = table.fresh_var();
            if let Type::Var(id) = table.resolve(var) {
                env.insert(g.text.clone(), id);
            }
        }
    }

    let mut args = Vec::new();
    for g in &obj.generics {
        if let Some(&id) = env.get(&g.text) {
            args.push(table.intern(Type::Var(id)));
        }
    }
    let _self_ty = table.intern(Type::Named {
        name: obj.name.text.clone(),
        args,
    });

    for method in &obj.methods {
        let mut generic_names = HashSet::new();
        let mut method_env = env.clone();
        for g in &method.generics {
            if !generic_names.insert(&g.text) {
                diags.push(Diagnostic::new(
                    ErrorKind::Type,
                    format!("duplicate generic parameter `{}`", g.text),
                    g.span.clone(),
                ));
            } else {
                let var = table.fresh_var();
                if let Type::Var(id) = table.resolve(var) {
                    method_env.insert(g.text.clone(), id);
                }
            }
        }

        let mut param_names = HashSet::new();
        for p in &method.params {
            if !param_names.insert(&p.name.text) {
                diags.push(Diagnostic::new(
                    ErrorKind::Type,
                    format!("duplicate parameter name `{}`", p.name.text),
                    p.name.span.clone(),
                ));
            }
        }

        let ret_id = if let Some(ret) = &method.ret {
            table.from_repr(ret, &method_env)
        } else {
            table.intern(Type::Unit)
        };

        check_block(table, &mut method_env, &method.body, ret_id, false, diags);
    }

    for g in &obj.generics {
        env.remove(&g.text);
    }
}

fn check_const(
    table: &mut TypeTable,
    _env: &mut HashMap<String, u32>,
    c: &ConstDef,
    diags: &mut Vec<Diagnostic>,
) {
    let _res = check_expr::check(table, &c.value).unwrap_or_else(|e| {
        diags.push(e);
        table.intern(Type::Never)
    });
}

fn check_block(
    table: &mut TypeTable,
    _env: &mut HashMap<String, u32>,
    block: &Block,
    ret_id: TypeId,
    is_main: bool,
    diags: &mut Vec<Diagnostic>,
) {
    let mut has_return = false;

    fn check_propagate(
        table: &mut TypeTable,
        expr: &Expr,
        ret_id: TypeId,
        diags: &mut Vec<Diagnostic>,
    ) {
        match &expr.kind {
            ExprKind::Propagate(inner) => {
                let _res = check_expr::check(table, inner).unwrap_or_else(|e| {
                    diags.push(e);
                    table.intern(Type::Never)
                });

                let ret_ty = table.resolve(ret_id);
                match ret_ty {
                    Type::ErrorUnion { .. } | Type::Never => {
                    }
                    _ => {
                        diags.push(Diagnostic::new(
                            ErrorKind::Type,
                            "E0400: cannot propagate error in a function that does not return an error union",
                            expr.span.clone(),
                        ));
                    }
                }
            }
            ExprKind::Binary { lhs, rhs, .. } => {
                check_propagate(table, lhs, ret_id, diags);
                check_propagate(table, rhs, ret_id, diags);
            }
            ExprKind::Unary { operand, .. } => {
                check_propagate(table, operand, ret_id, diags);
            }
            ExprKind::Call { callee, args } => {
                check_propagate(table, callee, ret_id, diags);
                for arg in args {
                    check_propagate(table, &arg.value, ret_id, diags);
                }
            }
            _ => {}
        }
    }

    for stmt in &block.stmts {
        match &stmt.kind {
            StmtKind::Return(opt_expr) => {
                has_return = true;
                if let Some(e) = opt_expr {
                    let ty_id = check_expr::check(table, e).unwrap_or_else(|err| {
                        diags.push(err);
                        table.intern(Type::Never)
                    });
                    if unify::unify(table, ty_id, ret_id).is_err() {
                        diags.push(Diagnostic::new(
                            ErrorKind::Type,
                            "return type mismatch",
                            e.span.clone(),
                        ));
                    }
                    check_propagate(table, e, ret_id, diags);
                } else {
                    let unit_ty = table.intern(Type::Unit);
                    let never_ty = table.intern(Type::Never);
                    if unify::unify(table, ret_id, never_ty).is_ok() || unify::unify(table, ret_id, unit_ty).is_ok() {

                    } else {
                        diags.push(Diagnostic::new(
                            ErrorKind::Type,
                            "missing return value",
                            stmt.span.clone(),
                        ));
                    }
                }
            }
            StmtKind::Assign { target, value } => {
                let val_ty = check_expr::check(table, value).unwrap_or_else(|e| {
                    diags.push(e);
                    table.intern(Type::Never)
                });
                let target_ty = check_expr::check(table, target).unwrap_or_else(|e| {
                    diags.push(e);
                    table.intern(Type::Never)
                });
                if unify::unify(table, target_ty, val_ty).is_err() {
                    diags.push(Diagnostic::new(
                        ErrorKind::Type,
                        "assignment type mismatch",
                        stmt.span.clone(),
                    ));
                }
                check_propagate(table, value, ret_id, diags);
            }
            StmtKind::Expr(e) => {
                let _res = check_expr::check(table, e).unwrap_or_else(|err| {
                    diags.push(err);
                    table.intern(Type::Never)
                });
                check_propagate(table, e, ret_id, diags);
            }
            _ => {}
        }
    }

    if !has_return && !is_main {
        let ty = table.resolve(ret_id);
        if ty != Type::Unit && ty != Type::Never {
            diags.push(Diagnostic::new(
                ErrorKind::Type,
                "missing return statement",
                block.span.clone(),
            ));
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::item::{Name, FnDef};

    fn name(n: &str) -> Name {
        Name::new(n, Span::new(0, 1))
    }

    #[test]
    fn test_main_returns_never() {
        let f = FnDef {
            name: name("main"),
            generics: vec![],
            params: vec![],
            ret: None,
            body: Block {
                stmts: vec![],
                span: Span::new(0, 1),
            },
        };
        let items = vec![Item {
            attrs: vec![],
            kind: ItemKind::Fn(f),
            span: Span::new(0, 1),
        }];
        let _diags = check_items(&items);
    }
}
""")
