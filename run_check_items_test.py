import sys

with open("src/types/check_item.rs", "w") as f:
    f.write("""use std::collections::{HashMap, HashSet};

use crate::ast::item::{FnDef, Item, ItemKind, ObjectDef, ConstDef};
use crate::ast::ty::{TypeKind, TypeRepr};
use crate::ast::expr::{Expr, ExprKind, Stmt, StmtKind, Block};
use crate::diag::{Diagnostic, ErrorKind, Span};
use crate::types::{check_expr, unify, Type, TypeId, TypeTable};

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
    // Check for duplicate generic params
    let mut generic_names = HashSet::new();
    for gen in &f.generics {
        if !generic_names.insert(&gen.text) {
            diags.push(Diagnostic::new(
                ErrorKind::Type,
                format!("duplicate generic parameter `{}`", gen.text),
                gen.span.clone(),
            ));
        } else {
            let var = table.fresh_var();
            if let Type::Var(id) = table.resolve(var) {
                env.insert(gen.text.clone(), id);
            }
        }
    }

    // Check for duplicate param names
    let mut param_names = HashSet::new();
    let mut param_types = Vec::new();
    for p in &f.params {
        if !param_names.insert(&p.name.text) {
            diags.push(Diagnostic::new(
                ErrorKind::Type,
                format!("duplicate parameter name `{}`", p.name.text),
                p.name.span.clone(),
            ));
        }
        let ty_id = table.from_repr(&p.ty, env);
        param_types.push(ty_id);
    }

    // Resolve return type
    let ret_id = if let Some(ret) = &f.ret {
        table.from_repr(ret, env)
    } else {
        table.intern(Type::Unit)
    };

    // Check main return
    if f.name.text == "main" {
        // main usually returns !
        let never = table.intern(Type::Never);
        if ret_id != never {
             // Not strictly specified by task, but useful
        }
    }

    // Check body (assignments etc)
    check_block(table, env, &f.body, ret_id, f.name.text == "main", diags);

    // Remove generics from env
    for gen in &f.generics {
        env.remove(&gen.text);
    }
}

fn check_object(
    table: &mut TypeTable,
    env: &mut HashMap<String, u32>,
    obj: &ObjectDef,
    diags: &mut Vec<Diagnostic>,
) {
    let mut generic_names = HashSet::new();
    for gen in &obj.generics {
        if !generic_names.insert(&gen.text) {
            diags.push(Diagnostic::new(
                ErrorKind::Type,
                format!("duplicate generic parameter `{}`", gen.text),
                gen.span.clone(),
            ));
        } else {
            let var = table.fresh_var();
            if let Type::Var(id) = table.resolve(var) {
                env.insert(gen.text.clone(), id);
            }
        }
    }

    // Construct the object type to use as `self`
    let mut args = Vec::new();
    for gen in &obj.generics {
        if let Some(&id) = env.get(&gen.text) {
            args.push(table.intern(Type::Var(id)));
        }
    }
    let self_ty = table.intern(Type::Named {
        name: obj.name.text.clone(),
        args,
    });

    for method in &obj.methods {
        // check method
        let mut generic_names = HashSet::new();
        let mut method_env = env.clone();
        for gen in &method.generics {
            if !generic_names.insert(&gen.text) {
                diags.push(Diagnostic::new(
                    ErrorKind::Type,
                    format!("duplicate generic parameter `{}`", gen.text),
                    gen.span.clone(),
                ));
            } else {
                let var = table.fresh_var();
                if let Type::Var(id) = table.resolve(var) {
                    method_env.insert(gen.text.clone(), id);
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
            if p.name.text == "self" {
                // verify self type?
            }
        }

        let ret_id = if let Some(ret) = &method.ret {
            table.from_repr(ret, &method_env)
        } else {
            table.intern(Type::Unit)
        };

        check_block(table, &mut method_env, &method.body, ret_id, false, diags);
    }

    for gen in &obj.generics {
        env.remove(&gen.text);
    }
}

fn check_const(
    table: &mut TypeTable,
    env: &mut HashMap<String, u32>,
    c: &ConstDef,
    diags: &mut Vec<Diagnostic>,
) {
    let _ = check_expr::check(table, &c.value).unwrap_or_else(|e| {
        diags.push(e);
        table.intern(Type::Never)
    });
}

fn check_block(
    table: &mut TypeTable,
    env: &mut HashMap<String, u32>,
    block: &Block,
    ret_id: TypeId,
    is_main: bool,
    diags: &mut Vec<Diagnostic>,
) {
    let mut has_return = false;

    // We need to recursively look for propagates!
    fn check_propagate(
        table: &mut TypeTable,
        expr: &Expr,
        ret_id: TypeId,
        diags: &mut Vec<Diagnostic>,
    ) {
        match &expr.kind {
            ExprKind::Propagate(inner) => {
                let res = check_expr::check(table, inner).unwrap_or_else(|e| {
                    diags.push(e);
                    table.intern(Type::Never)
                });

                let ret_ty = table.resolve(ret_id);
                match ret_ty {
                    Type::ErrorUnion { err: err_b, .. } | Type::Never => {
                        // Fine, if Never we don't care, if ErrorUnion we'd need to unify with err_b
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
            // Add other cases...
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
                    if ret_id != unit_ty && ret_id != never_ty {
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
                let _ = check_expr::check(table, e).unwrap_or_else(|err| {
                    diags.push(err);
                    table.intern(Type::Never)
                });
                check_propagate(table, e, ret_id, diags);
            }
            // Add more statement checks...
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
""")
