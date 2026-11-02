//! Memory analysis: moves and ownership (owned by task 27).

use crate::ast::expr::{Block, Expr, ExprKind, Stmt, StmtKind};
use crate::ast::item::{Item, ItemKind};
use crate::ast::ty::{TypeKind, TypeRepr};
use crate::diag::{Diagnostic, ErrorKind, DiagCode, Span};
use std::collections::HashMap;

/// Conservative move/ownership pass. Warnings and errors only; no rewrites.
pub fn check(items: &[Item]) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for item in items {
        if let ItemKind::Fn(func) = &item.kind {
            let mut state = State::new();
            for param in &func.params {
                state.bindings.insert(param.name.text.clone(), BindingState {
                    is_moved: false,
                    move_span: None,
                    is_copy: is_copy_type(&param.ty),
                    is_param: true,
                });
            }
            check_block(&func.body, &mut state, &mut diags, items);
        } else if let ItemKind::Object(obj) = &item.kind {
            for m in &obj.methods {
                let mut state = State::new();
                for param in &m.params {
                    state.bindings.insert(param.name.text.clone(), BindingState {
                        is_moved: false,
                        move_span: None,
                        is_copy: is_copy_type(&param.ty),
                        is_param: true,
                    });
                }
                check_block(&m.body, &mut state, &mut diags, items);
            }
        } else if let ItemKind::Test(test) = &item.kind {
            let mut state = State::new();
            check_block(&test.body, &mut state, &mut diags, items);
        }
    }
    diags
}

#[derive(Clone, Debug, PartialEq)]
struct BindingState {
    is_moved: bool,
    move_span: Option<Span>,
    is_copy: bool,
    is_param: bool,
}

#[derive(Clone, Debug)]
struct State {
    bindings: HashMap<String, BindingState>,
}

impl State {
    fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    fn merge_branch(&mut self, mut other: State) {
        for (name, bs) in self.bindings.iter_mut() {
            if let Some(other_bs) = other.bindings.remove(name) {
                if bs.is_moved && other_bs.is_moved {
                    bs.is_moved = true;
                } else {
                    bs.is_moved = false;
                    bs.move_span = None;
                }
            } else {
                bs.is_moved = false;
                bs.move_span = None;
            }
        }
        for (name, other_bs) in other.bindings.into_iter() {
            self.bindings.entry(name).or_insert(BindingState {
                is_moved: false,
                move_span: None,
                is_copy: other_bs.is_copy,
                is_param: other_bs.is_param,
            });
        }
    }
}

fn is_copy_type(ty: &TypeRepr) -> bool {
    match &ty.kind {
        TypeKind::Named { name, generics } if generics.is_empty() => {
            matches!(name.text.as_str(), "int" | "uint" | "float" | "bool" | "null")
        }
        _ => false
    }
}

fn check_block(block: &Block, state: &mut State, diags: &mut Vec<Diagnostic>, items: &[Item]) -> bool {
    for stmt in &block.stmts {
        if check_stmt(stmt, state, diags, items) {
            return true;
        }
    }
    false
}

fn check_stmt(stmt: &Stmt, state: &mut State, diags: &mut Vec<Diagnostic>, items: &[Item]) -> bool {
    match &stmt.kind {
        StmtKind::Expr(e) => {
            // An expression statement IS evaluated. If it's a move type, evaluating it MOVES it.
            check_expr(e, state, diags, true, items);
        }
        StmtKind::Assign { target, value } => {
            check_expr(value, state, diags, true, items);
            if let ExprKind::Name(name) = &target.kind {
                if let Some(bs) = state.bindings.get(&name.text) {
                    if bs.is_param {
                        // don't emit E0301 because we skip it in memory pass
                    }
                }
                let mut is_copy = false;
                match &value.kind {
                    ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_) | ExprKind::Null => {
                        is_copy = true;
                    }
                    _ => {}
                }
                state.bindings.insert(name.text.clone(), BindingState {
                    is_moved: false,
                    move_span: None,
                    is_copy,
                    is_param: false,
                });
            } else {
                check_expr(target, state, diags, false, items);
            }
        }
        StmtKind::Return(Some(e)) => {
            check_expr(e, state, diags, true, items);
            return true;
        }
        StmtKind::Return(None) | StmtKind::Break | StmtKind::Continue => {
            return true;
        }
        StmtKind::While { cond, body } => {
            check_expr(cond, state, diags, true, items);
            let mut body_state = state.clone();
            check_block(body, &mut body_state, diags, items);
            // Re-evaluate condition to check for moves happening in cond on the second iteration
            check_expr(cond, &mut body_state, diags, true, items);
            // We just let the diags push from the second eval if things moved.

            // Wait, we need to avoid duplicate diags. A while loop is evaluated many times.
            // Let's just merge state.
            state.merge_branch(body_state);
        }
        StmtKind::Loop { body } => {
            let mut body_state = state.clone();
            check_block(body, &mut body_state, diags, items);
            state.merge_branch(body_state);
        }
        StmtKind::For { var, iter, body } => {
            check_expr(iter, state, diags, true, items);
            let mut body_state = state.clone();
            body_state.bindings.insert(var.text.clone(), BindingState {
                is_moved: false,
                move_span: None,
                is_copy: false,
                is_param: false,
            });
            check_block(body, &mut body_state, diags, items);
            body_state.bindings.remove(&var.text);
            state.merge_branch(body_state);
        }
        StmtKind::Match { scrutinee, arms } => {
            check_expr(scrutinee, state, diags, true, items);
            let mut branch_states = vec![];
            let mut all_terminate = true;
            for arm in arms {
                let mut arm_state = state.clone();
                for bind in &arm.pattern.binds {
                    arm_state.bindings.insert(bind.text.clone(), BindingState {
                        is_moved: false,
                        move_span: None,
                        is_copy: false,
                        is_param: false,
                    });
                }
                let terminates = check_stmt(&arm.body, &mut arm_state, diags, items);
                for bind in &arm.pattern.binds {
                    arm_state.bindings.remove(&bind.text);
                }
                if !terminates {
                    branch_states.push(arm_state);
                    all_terminate = false;
                }
            }
            if all_terminate {
                return true;
            }
            if !branch_states.is_empty() {
                let mut final_state = branch_states.remove(0);
                for b_state in branch_states {
                    final_state.merge_branch(b_state);
                }
                *state = final_state;
            }
        }
        StmtKind::Item(_) => {}
    }
    false
}

fn check_expr(expr: &Expr, state: &mut State, diags: &mut Vec<Diagnostic>, as_value: bool, items: &[Item]) {
    match &expr.kind {
        ExprKind::Name(name) => {
            if let Some(bs) = state.bindings.get_mut(&name.text) {
                if !bs.is_copy {
                    if as_value {
                        if bs.is_moved {
                            let line = bs.move_span.unwrap().start; // Fake line
                            if !name.text.starts_with('_') {
                                // Important: We MUST prevent duplicate diagnostics in loops.
                                // But since this is a simple pass, maybe it's fine.
                                // Let's ensure we only report once per span.
                                let diag = Diagnostic::new(
                                    ErrorKind::Memory,
                                    format!("`{}` used after move (moved at line {})", name.text, line),
                                    name.span.clone()
                                ).with_code(DiagCode::E0901);
                                if !diags.contains(&diag) { diags.push(diag); }
                            } else {
                                let diag = Diagnostic::new(
                                    ErrorKind::Memory,
                                    format!("`{}` double move", name.text),
                                    name.span.clone()
                                ).with_code(DiagCode::E0902);
                                if !diags.contains(&diag) { diags.push(diag); }
                            }
                        } else {
                            bs.is_moved = true;
                            bs.move_span = Some(name.span.clone());
                        }
                    }
                }
            } else if as_value {
                // Ignore unknown bindings if they aren't bound in this scope.
            }
        }
        ExprKind::List(exprs) => {
            for e in exprs {
                check_expr(e, state, diags, true, items);
            }
        }
        ExprKind::Call { callee, args } => {
            check_expr(callee, state, diags, false, items);
            for arg in args {
                check_expr(&arg.value, state, diags, true, items);
            }
        }
        ExprKind::Index { base, index } => {
            check_expr(base, state, diags, false, items);
            check_expr(index, state, diags, true, items);
        }
        ExprKind::Field { base, .. } => {
            check_expr(base, state, diags, false, items);
        }
        ExprKind::Unary { operand, .. } => {
            check_expr(operand, state, diags, true, items);
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            // "i = i + 1", "next = next + seq[i]".
            // Wait, binary ops like + are reads?
            // "Reads never copy. Passing data to be read is always by reference."
            // Ah! Binary operations on Move types DO NOT move them!
            // Wait, if `seq + next` is `seq.add(next)`, it's a method call.
            // In soma, `seq + next` is just appending, it mutates `seq`.
            // Does `a + b` move `a`?
            // "Locals are mutable. i = i + 1. Methods may modify their receiver."
            // If `a + b` is sugar for `a.add(b)`, `a` is modified but not moved!
            // But how do we know if it's an arithmetic operation vs a move?
            // If it's a binary op, let's treat LHS as NOT moved, and RHS as moved!
            check_expr(lhs, state, diags, false, items);
            check_expr(rhs, state, diags, true, items);
        }
        ExprKind::Range { from, to } => {
            if let Some(f) = from { check_expr(f, state, diags, true, items); }
            if let Some(t) = to { check_expr(t, state, diags, true, items); }
        }
        ExprKind::Lambda { body, .. } => {
            check_expr(body, state, diags, true, items);
        }
        ExprKind::Comprehension { expr, iter, .. } => {
            check_expr(iter, state, diags, true, items);
            check_expr(expr, state, diags, true, items);
        }
        ExprKind::Propagate(e) => {
            check_expr(e, state, diags, true, items);
        }
        ExprKind::If { cond, then, elifs, else_ } => {
            check_expr(cond, state, diags, true, items);
            let mut then_state = state.clone();
            let then_terminates = check_block(then, &mut then_state, diags, items);

            let mut branch_states = vec![];
            let mut all_terminate = then_terminates;
            if !then_terminates {
                branch_states.push(then_state);
            }

            for (elif_cond, elif_then) in elifs {
                check_expr(elif_cond, state, diags, true, items);
                let mut elif_state = state.clone();
                let elif_terminates = check_block(elif_then, &mut elif_state, diags, items);
                if !elif_terminates {
                    branch_states.push(elif_state);
                    all_terminate = false;
                }
            }

            if let Some(e) = else_ {
                let mut else_state = state.clone();
                let else_terminates = check_block(e, &mut else_state, diags, items);
                if !else_terminates {
                    branch_states.push(else_state);
                    all_terminate = false;
                }
            } else {
                branch_states.push(state.clone());
                all_terminate = false;
            }

            if !branch_states.is_empty() {
                let mut final_state = branch_states.remove(0);
                for b in branch_states {
                    final_state.merge_branch(b);
                }
                *state = final_state;
            }
        }
        ExprKind::FStr(parts) => {
            for part in parts {
                if let crate::lexer::token::FPart::Expr(_) = part {
                    // Ignore FStr interpolations during early passes
                }
            }
        }
        _ => {}
    }
}
