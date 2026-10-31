//! Tree-walking evaluator (owned by task 15).

use std::collections::BTreeMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::ast::expr::{BinOp, Block, Expr, ExprKind, Stmt, StmtKind, UnOp};
use crate::ast::item::ObjectDef;
use crate::diag::{Diagnostic, ErrorKind, Span};
use crate::interp::value::Value;
use crate::interp::RunOutput;
use crate::lexer::token::FPart;

const MAX_STEPS: usize = 10_000_000;

pub struct Context {
    pub output: RunOutput,
    pub globals: BTreeMap<String, Value>,
    pub objects: BTreeMap<String, ObjectDef>,
    pub steps: usize,
}

impl Context {
    pub fn new() -> Self {
        Self {
            output: RunOutput::default(),
            globals: BTreeMap::new(),
            objects: BTreeMap::new(),
            steps: 0,
        }
    }

    pub fn tick(&mut self, span: Span) -> Result<(), Diagnostic> {
        self.steps += 1;
        if self.steps > MAX_STEPS {
            Err(Diagnostic::new(ErrorKind::Other, "[E0600] runaway loop detected (>10M steps)", span))
        } else {
            Ok(())
        }
    }

    pub fn runtime_error(&self, msg: impl Into<String>, span: Span) -> Diagnostic {
        Diagnostic::new(ErrorKind::Other, format!("[E0600] {}", msg.into()), span)
    }
}

#[derive(Clone)]
pub struct Env {
    pub vars: Rc<RefCell<BTreeMap<String, Value>>>,
    pub parent: Option<Box<Env>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            vars: Rc::new(RefCell::new(BTreeMap::new())),
            parent: None,
        }
    }

    pub fn with_parent(parent: Env) -> Self {
        Self {
            vars: Rc::new(RefCell::new(BTreeMap::new())),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.vars.borrow().get(name) {
            Some(v.clone())
        } else if let Some(p) = &self.parent {
            p.get(name)
        } else {
            None
        }
    }

    pub fn define(&self, name: String, value: Value) {
        self.vars.borrow_mut().insert(name, value);
    }

    pub fn set(&self, name: &str, value: Value) -> bool {
        if self.vars.borrow().contains_key(name) {
            self.vars.borrow_mut().insert(name.to_string(), value);
            true
        } else if let Some(p) = &self.parent {
            p.set(name, value)
        } else {
            false
        }
    }
}

pub enum ControlFlow {
    Return(Value),
    Break,
    Continue,
    Error(Diagnostic),
}

impl From<Diagnostic> for ControlFlow {
    fn from(e: Diagnostic) -> Self {
        ControlFlow::Error(e)
    }
}

fn to_string_val(ctx: &Context, v: &Value, span: Span) -> Result<String, Diagnostic> {
    match v {
        Value::Int(i) => Ok(i.to_string()),
        Value::Float(f) => Ok(f.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Str(s) => Ok(s.clone()),
        Value::Null => Ok(String::from("null")),
        Value::Unit => Ok(String::from("!")),
        _ => Err(ctx.runtime_error(format!("cannot convert {} to string", v.type_name()), span)),
    }
}

pub fn eval_expr(ctx: &mut Context, env: &Env, expr: &Expr) -> Result<Value, ControlFlow> {
    ctx.tick(expr.span)?;
    match &expr.kind {
        ExprKind::Int(v) => Ok(Value::Int(*v)),
        ExprKind::Float(v) => Ok(Value::Float(*v)),
        ExprKind::Str(v) => Ok(Value::Str(v.clone())),
        ExprKind::FStr(parts) => {
            let mut s = String::new();
            for part in parts {
                match part {
                    FPart::Text(t) => s.push_str(t),
                    FPart::Expr(e_str) => {
                        let dummy_src = format!("fn __fstr() -> any {{\n  return {}\n}}", e_str);
                        let parsed = crate::driver::parse_source("fstr", &dummy_src).map_err(|e| ctx.runtime_error(e.message, expr.span))?;
                        let lowered = crate::desugar::lower(parsed.items);
                        let mut val = Value::Unit;
                        if let Some(crate::ast::item::Item { kind: crate::ast::item::ItemKind::Fn(f), .. }) = lowered.first() {
                            if let Some(crate::ast::expr::Stmt { kind: crate::ast::expr::StmtKind::Return(Some(e)), .. }) = f.body.stmts.first() {
                                val = eval_expr(ctx, env, e)?;
                            }
                        }
                        s.push_str(&to_string_val(ctx, &val, expr.span)?);
                    }
                }
            }
            Ok(Value::Str(s))
        },
        ExprKind::Bool(v) => Ok(Value::Bool(*v)),
        ExprKind::Null => Ok(Value::Null),
        ExprKind::Name(n) => {
            if let Some(v) = env.get(&n.text) {
                Ok(v)
            } else if let Some(v) = ctx.globals.get(&n.text) {
                Ok(v.clone())
            } else if let Some(_) = ctx.objects.get(&n.text) {
                Ok(Value::BuiltinFn(n.text.clone()))
            } else {
                let first_char = n.text.chars().next().unwrap_or('a');
                if first_char.is_uppercase() {
                    Ok(Value::BuiltinFn(n.text.clone()))
                } else {
                    Err(ctx.runtime_error(format!("undefined name '{}'", n.text), expr.span).into())
                }
            }
        },
        ExprKind::SelfExpr => {
            if let Some(v) = env.get("self") {
                Ok(v)
            } else {
                Err(ctx.runtime_error("'self' used outside of method", expr.span).into())
            }
        },
        ExprKind::List(items) => {
            let mut res = Vec::new();
            for item in items {
                res.push(eval_expr(ctx, env, item)?);
            }
            Ok(Value::List(Rc::new(RefCell::new(res))))
        },
        ExprKind::Call { callee, args } => {
            let mut is_method = false;
            let mut self_obj = None;
            let mut method_name = None;
            if let ExprKind::Field { base, name } = &callee.kind {
                if let Ok(obj) = eval_expr(ctx, env, base) {
                    self_obj = Some(obj);
                    method_name = Some(name.text.clone());
                    is_method = true;
                }
            }

            let func = if is_method { Value::Unit } else { eval_expr(ctx, env, callee)? };

            let mut evaled_args = Vec::new();
            let mut named_args = BTreeMap::new();
            for arg in args {
                let v = eval_expr(ctx, env, &arg.value)?;
                if let Some(n) = &arg.name {
                    named_args.insert(n.text.clone(), v.clone());
                }
                evaled_args.push(v);
            }
            if is_method {
                let name = method_name.unwrap();
                let obj = self_obj.unwrap();
                if let Value::List(l) = &obj {
                    if name == "len" {
                        return Ok(Value::Int(l.borrow().len() as i64));
                    } else if name == "add" {
                        l.borrow_mut().push(evaled_args[0].clone());
                        return Ok(Value::Unit);
                    } else if name == "extend" {
                        if let Value::List(other) = &evaled_args[0] {
                            let items = other.borrow().clone();
                            l.borrow_mut().extend(items.into_iter());
                            return Ok(Value::Unit);
                        } else {
                            return Err(ctx.runtime_error("extend expects list", expr.span).into());
                        }
                    } else {
                        return Err(ctx.runtime_error(format!("unknown list method {}", name), expr.span).into());
                    }
                } else if let Value::Str(s) = &obj {
                    if name == "len" {
                        return Ok(Value::Int(s.len() as i64));
                    } else if name == "contains" {
                        if let Value::Str(sub) = &evaled_args[0] {
                            return Ok(Value::Bool(s.contains(sub)));
                        } else {
                            return Err(ctx.runtime_error("contains expects string", expr.span).into());
                        }
                    } else if name == "lower" {
                        return Ok(Value::Str(s.to_lowercase()));
                    } else {
                        return Err(ctx.runtime_error(format!("unknown string method {}", name), expr.span).into());
                    }
                } else if let Value::Object(_) = &obj {
                    let mut found_method = None;
                    for obj_def in ctx.objects.values() {
                        for m in &obj_def.methods {
                            if m.name.text == name {
                                found_method = Some(m.clone());
                                break;
                            }
                        }
                        if found_method.is_some() { break; }
                    }
                    if let Some(m) = found_method {
                        let new_env = Env::with_parent(env.clone());
                        new_env.define("self".to_string(), obj.clone());
                        if m.params.len() != evaled_args.len() {
                            return Err(ctx.runtime_error("arity mismatch", expr.span).into());
                        }
                        for (param, arg_val) in m.params.iter().zip(evaled_args) {
                            new_env.define(param.name.text.clone(), arg_val);
                        }
                        match eval_block(ctx, &new_env, &m.body) {
                            Ok(_) => return Ok(Value::Unit),
                            Err(ControlFlow::Return(v)) => return Ok(v),
                            Err(e) => return Err(e),
                        }
                    } else {
                        if let Value::Object(fields) = obj {
                            if let Some(Value::Fn(f)) = fields.borrow().get(&name) {
                                let new_env = Env::with_parent(env.clone());
                                if f.params.len() != evaled_args.len() {
                                    return Err(ctx.runtime_error("arity mismatch", expr.span).into());
                                }
                                for (param, arg_val) in f.params.iter().zip(evaled_args.clone()) {
                                    new_env.define(param.name.text.clone(), arg_val);
                                }
                                match eval_block(ctx, &new_env, &f.body) {
                                    Ok(_) => return Ok(Value::Unit),
                                    Err(ControlFlow::Return(v)) => return Ok(v),
                                    Err(e) => return Err(e),
                                }
                            }
                        }
                        return Err(ctx.runtime_error(format!("method {} not found", name), expr.span).into());
                    }
                }
            }

            match func {
                Value::Fn(f) => {
                    let new_env = Env::with_parent(env.clone());
                    if f.params.len() != evaled_args.len() {
                        return Err(ctx.runtime_error("arity mismatch", expr.span).into());
                    }
                    for (param, arg_val) in f.params.iter().zip(evaled_args) {
                        new_env.define(param.name.text.clone(), arg_val);
                    }
                    match eval_block(ctx, &new_env, &f.body) {
                        Ok(_) => Ok(Value::Unit),
                        Err(ControlFlow::Return(v)) => Ok(v),
                        Err(e) => Err(e),
                    }
                },
                Value::BuiltinFn(name) => {
                    if name == "output" {
                        if evaled_args.len() != 1 {
                            return Err(ctx.runtime_error("output requires 1 argument", expr.span).into());
                        }
                        let s = to_string_val(ctx, &evaled_args[0], expr.span)?;
                        ctx.output.stdout.push_str(&s);
                        ctx.output.stdout.push_str("\n");
                        Ok(Value::Unit)
                    } else if name == "input" {
                        Err(ctx.runtime_error("input is an error", expr.span).into())
                    } else if name.starts_with("string.") {
                        let obj = &evaled_args[0];
                        if let Value::Str(s) = obj {
                            if name == "string.len" {
                                Ok(Value::Int(s.len() as i64))
                            } else if name == "string.contains" {
                                if let Value::Str(sub) = &evaled_args[1] {
                                    Ok(Value::Bool(s.contains(sub)))
                                } else {
                                    Err(ctx.runtime_error("contains expects string", expr.span).into())
                                }
                            } else if name == "string.lower" {
                                Ok(Value::Str(s.to_lowercase()))
                            } else {
                                Err(ctx.runtime_error(format!("unknown string method {}", name), expr.span).into())
                            }
                        } else {
                            Err(ctx.runtime_error("expected string", expr.span).into())
                        }
                    } else if name.starts_with("list.") {
                        let obj = &evaled_args[0];
                        if let Value::List(l) = obj {
                            if name == "list.len" {
                                Ok(Value::Int(l.borrow().len() as i64))
                            } else if name == "list.add" {
                                l.borrow_mut().push(evaled_args[1].clone());
                                Ok(Value::Unit)
                            } else if name == "list.extend" {
                                if let Value::List(other) = &evaled_args[1] {
                                    let items = other.borrow().clone();
                                    l.borrow_mut().extend(items.into_iter());
                                    Ok(Value::Unit)
                                } else {
                                    Err(ctx.runtime_error("extend expects list", expr.span).into())
                                }
                            } else {
                                Err(ctx.runtime_error(format!("unknown list method {}", name), expr.span).into())
                            }
                        } else {
                            Err(ctx.runtime_error("expected list", expr.span).into())
                        }
                    } else if name == "env.args" {
                        Ok(Value::List(Rc::new(RefCell::new(vec![]))))
                    } else if let Some(obj_def) = ctx.objects.get(&name).cloned() {
                        let mut fields = BTreeMap::new();
                        if !named_args.is_empty() {
                            for (k, v) in named_args {
                                fields.insert(k, v);
                            }
                        } else {
                            if obj_def.fields.len() != evaled_args.len() {
                                return Err(ctx.runtime_error("arity mismatch in constructor", expr.span).into());
                            }
                            for (field_def, val) in obj_def.fields.iter().zip(evaled_args) {
                                fields.insert(field_def.name.text.clone(), val);
                            }
                        }
                        Ok(Value::Object(Rc::new(RefCell::new(fields))))
                    } else if name.chars().next().unwrap_or('a').is_uppercase() {
                        Ok(Value::Enum(name.to_string(), evaled_args))
                    } else {
                        Err(ctx.runtime_error(format!("unknown builtin {}", name), expr.span).into())
                    }
                },
                _ => {
                    Err(ctx.runtime_error(String::from("not callable"), expr.span).into())
                }
            }
        },
        ExprKind::Field { base, name } => {
            let obj = eval_expr(ctx, env, base)?;
            if let Value::Object(fields) = &obj {
                if let Some(val) = fields.borrow().get(&name.text) {
                    return Ok(val.clone());
                }
            }
            Err(ctx.runtime_error(String::from("invalid field access"), expr.span).into())
        },
        ExprKind::Index { base, index } => {
            let b = eval_expr(ctx, env, base)?;
            let i = eval_expr(ctx, env, index)?;
            match (&b, &i) {
                (Value::List(l), Value::Int(idx)) => {
                    let list = l.borrow();
                    if *idx < 0 || *idx >= list.len() as i64 {
                        return Err(ctx.runtime_error(String::from("index out of bounds"), expr.span).into());
                    }
                    Ok(list[*idx as usize].clone())
                },
                _ => Err(ctx.runtime_error(String::from("invalid index operation"), expr.span).into()),
            }
        },
        ExprKind::Binary { op, lhs, rhs } => {
            if *op == BinOp::And || *op == BinOp::Or {
                let l = eval_expr(ctx, env, lhs)?;
                if *op == BinOp::And {
                    if l == Value::Bool(false) { return Ok(Value::Bool(false)); }
                    return eval_expr(ctx, env, rhs);
                } else {
                    if l == Value::Bool(true) { return Ok(Value::Bool(true)); }
                    return eval_expr(ctx, env, rhs);
                }
            }
            let l = eval_expr(ctx, env, lhs)?;
            let r = eval_expr(ctx, env, rhs)?;
            match (op, &l, &r) {
                (BinOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (BinOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                (BinOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                (BinOp::Div, Value::Int(a), Value::Int(b)) => {
                    if *b == 0 { Err(ctx.runtime_error(String::from("division by zero"), expr.span).into()) } else { Ok(Value::Int(a / b)) }
                },
                (BinOp::Rem, Value::Int(a), Value::Int(b)) => {
                    if *b == 0 { Err(ctx.runtime_error(String::from("division by zero"), expr.span).into()) } else { Ok(Value::Int(a % b)) }
                },
                (BinOp::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (BinOp::Sub, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (BinOp::Mul, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (BinOp::Div, Value::Float(a), Value::Float(b)) => {
                    if *b == 0.0 { Err(ctx.runtime_error(String::from("division by zero"), expr.span).into()) } else { Ok(Value::Float(a / b)) }
                },
                (BinOp::Add, Value::Str(a), Value::Str(b)) => {
                    let mut s = a.clone();
                    s.push_str(b);
                    Ok(Value::Str(s))
                },
                (BinOp::Eq, _, _) => Ok(Value::Bool(l == r)),
                (BinOp::Ne, _, _) => Ok(Value::Bool(l != r)),
                (BinOp::Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                (BinOp::Le, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                (BinOp::Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                (BinOp::Ge, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                (BinOp::Lt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
                (BinOp::Le, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
                (BinOp::Gt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
                (BinOp::Ge, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
                _ => Err(ctx.runtime_error(format!("invalid binary operands: {:?} and {:?}", l, r), expr.span).into()),
            }
        },
        ExprKind::Unary { op, operand } => {
            let v = eval_expr(ctx, env, operand)?;
            match (op, v) {
                (UnOp::Neg, Value::Int(i)) => Ok(Value::Int(-i)),
                (UnOp::Neg, Value::Float(f)) => Ok(Value::Float(-f)),
                (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                _ => Err(ctx.runtime_error(String::from("invalid unary operand"), expr.span).into()),
            }
        },
        ExprKind::If { cond, then, elifs, else_ } => {
            let c = eval_expr(ctx, env, cond)?;
            if c == Value::Bool(true) {
                let new_env = Env::with_parent(env.clone());
                match eval_block(ctx, &new_env, then) {
                    Ok(_) => Ok(Value::Unit),
                    Err(e) => Err(e),
                }
            } else {
                for (elif_cond, elif_block) in elifs {
                    let ec = eval_expr(ctx, env, elif_cond)?;
                    if ec == Value::Bool(true) {
                        let new_env = Env::with_parent(env.clone());
                        return match eval_block(ctx, &new_env, elif_block) {
                            Ok(_) => Ok(Value::Unit),
                            Err(e) => Err(e),
                        };
                    }
                }
                if let Some(eblock) = else_ {
                    let new_env = Env::with_parent(env.clone());
                    match eval_block(ctx, &new_env, eblock) {
                        Ok(_) => Ok(Value::Unit),
                        Err(e) => Err(e),
                    }
                } else {
                    Ok(Value::Unit)
                }
            }
        },
        ExprKind::Propagate(e) => {
            eval_expr(ctx, env, e)
        },
        ExprKind::Lambda { params, body } => {
            let mut f = crate::ast::item::FnDef {
                name: crate::ast::item::Name { text: "lambda".to_string(), span: expr.span },
                generics: vec![],
                params: vec![],
                ret: None,
                body: Block { stmts: vec![Stmt { kind: StmtKind::Return(Some(*body.clone())), span: expr.span }], span: expr.span },
            };
            for p in params {
                f.params.push(crate::ast::item::Param { name: p.clone(), ty: crate::ast::ty::TypeRepr { kind: crate::ast::ty::TypeKind::Named { name: crate::ast::item::Name { text: "any".to_string(), span: expr.span }, generics: vec![] }, span: expr.span } });
            }
            Ok(Value::Fn(f))
        },
        ExprKind::Comprehension { expr: _, var: _, iter: _ } => {
            Err(ctx.runtime_error(String::from("comprehensions should be lowered"), expr.span).into())
        },
        ExprKind::Range { from: _, to: _ } => {
            Err(ctx.runtime_error(String::from("ranges should be lowered"), expr.span).into())
        }
    }
}

pub fn eval_block(ctx: &mut Context, env: &Env, block: &Block) -> Result<(), ControlFlow> {
    for stmt in &block.stmts {
        eval_stmt(ctx, env, stmt)?;
    }
    Ok(())
}

pub fn eval_stmt(ctx: &mut Context, env: &Env, stmt: &Stmt) -> Result<(), ControlFlow> {
    ctx.tick(stmt.span)?;
    match &stmt.kind {
        StmtKind::Expr(e) => {
            eval_expr(ctx, env, e)?;
        },
        StmtKind::Assign { target, value } => {
            let val = eval_expr(ctx, env, value)?;
            match &target.kind {
                ExprKind::Name(n) => {
                    if !env.set(&n.text, val.clone()) {
                        env.define(n.text.clone(), val);
                    }
                },
                ExprKind::Field { base, name } => {
                    let b = eval_expr(ctx, env, base)?;
                    if let Value::Object(fields) = b {
                        fields.borrow_mut().insert(name.text.clone(), val);
                    } else {
                        return Err(ctx.runtime_error(String::from("invalid field assignment target"), target.span).into());
                    }
                },
                ExprKind::Index { base, index } => {
                    let b = eval_expr(ctx, env, base)?;
                    let i = eval_expr(ctx, env, index)?;
                    match (b, i) {
                        (Value::List(l), Value::Int(idx)) => {
                            let mut list = l.borrow_mut();
                            if idx < 0 || idx >= list.len() as i64 {
                                return Err(ctx.runtime_error(String::from("index out of bounds"), target.span).into());
                            }
                            list[idx as usize] = val;
                        },
                        _ => return Err(ctx.runtime_error(String::from("invalid index assignment target"), target.span).into()),
                    }
                },
                _ => return Err(ctx.runtime_error(String::from("invalid assignment target"), target.span).into()),
            }
        },
        StmtKind::While { cond, body } => {
            loop {
                let c = eval_expr(ctx, env, cond)?;
                if c == Value::Bool(false) { break; }
                if c != Value::Bool(true) {
                    return Err(ctx.runtime_error(String::from("condition must be bool"), cond.span).into());
                }
                let new_env = Env::with_parent(env.clone());
                match eval_block(ctx, &new_env, body) {
                    Ok(_) => {}
                    Err(ControlFlow::Break) => break,
                    Err(ControlFlow::Continue) => continue,
                    Err(e) => return Err(e),
                }
            }
        },
        StmtKind::Loop { body } => {
            loop {
                let new_env = Env::with_parent(env.clone());
                match eval_block(ctx, &new_env, body) {
                    Ok(_) => {}
                    Err(ControlFlow::Break) => break,
                    Err(ControlFlow::Continue) => continue,
                    Err(e) => return Err(e),
                }
            }
        },
        StmtKind::Match { scrutinee, arms } => {
            let s = eval_expr(ctx, env, scrutinee)?;
            let mut matched = false;
            for arm in arms {
                if arm.pattern.name.text == "_" {
                    let new_env = Env::with_parent(env.clone());
                    match eval_stmt(ctx, &new_env, &arm.body) {
                        Ok(_) => {}
                        Err(e) => return Err(e),
                    }
                    matched = true;
                    break;
                }

                if let Value::Enum(ref name, ref fields) = s {
                    if name == &arm.pattern.name.text {
                        let new_env = Env::with_parent(env.clone());
                        if fields.len() != arm.pattern.binds.len() {
                            return Err(ctx.runtime_error(String::from("match bind arity mismatch"), arm.span).into());
                        }
                        for (bind, val) in arm.pattern.binds.iter().zip(fields.iter()) {
                            new_env.define(bind.text.clone(), val.clone());
                        }
                        match eval_stmt(ctx, &new_env, &arm.body) {
                            Ok(_) => {}
                            Err(e) => return Err(e),
                        }
                        matched = true;
                        break;
                    }
                }
            }
            if !matched {
                return Err(ctx.runtime_error(String::from("non-exhaustive match"), stmt.span).into());
            }
        },
        StmtKind::Break => return Err(ControlFlow::Break),
        StmtKind::Continue => return Err(ControlFlow::Continue),
        StmtKind::Return(opt_e) => {
            let v = match opt_e {
                Some(e) => eval_expr(ctx, env, e)?,
                None => Value::Unit,
            };
            return Err(ControlFlow::Return(v));
        },
        StmtKind::For { var: _, iter: _, body: _ } => {
            return Err(ctx.runtime_error(String::from("for should be lowered"), stmt.span).into());
        },
        StmtKind::Item(_) => {}, // skip nested items
    }
    Ok(())
}
