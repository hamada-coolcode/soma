use crate::ast::expr::{Expr, ExprKind, Block, Stmt, StmtKind, UnOp, BinOp};
use crate::ast::item::{Item, ItemKind, FnDef, ObjectDef};
use crate::diag::{Diagnostic, ErrorKind, Span};
use crate::lexer::token::FPart;
use super::value::Value;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    pub vars: HashMap<String, Value>,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            vars: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.vars.get(name) {
            Some(val.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn set(&mut self, name: String, value: Value) -> bool {
        if self.vars.contains_key(&name) {
            self.vars.insert(name, value);
            true
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().set(name, value)
        } else {
            false
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.vars.insert(name, value);
    }
}

pub enum Flow {
    Normal,
    /// Value of the last expression in a block (used by if/else expressions).
    Value(Value),
    Return(Value),
    Break,
    Continue,
}

pub struct Evaluator {
    /// A `return` executed inside an expression-position block.
    pub pending_return: Option<Value>,
    pub env: Rc<RefCell<Environment>>,
    pub stdout: String,
    pub steps: usize,
    /// Object definitions by type name, for construction and method dispatch.
    pub objects: HashMap<String, ObjectDef>,
    /// Variant names that belong to an `error` declaration.
    pub error_variants: HashSet<String>,
    /// Payload arity of every enum variant.
    pub variant_arity: HashMap<String, usize>,
}

impl Evaluator {
    pub fn new() -> Self {
        let env = Rc::new(RefCell::new(Environment::new()));
        env.borrow_mut().define("output".to_string(), Value::Builtin("output"));
        env.borrow_mut().define("input".to_string(), Value::Builtin("input"));
        env.borrow_mut().define("f".to_string(), Value::Builtin("f"));
        env.borrow_mut().define("is_error".to_string(), Value::Builtin("is_error"));

        let mut env_obj = std::collections::BTreeMap::new();
        env_obj.insert("args".to_string(), Value::List(Rc::new(RefCell::new(vec![]))));
        env.borrow_mut().define("env".to_string(), Value::Object(Rc::new(RefCell::new(env_obj))));

        Self {
            pending_return: None,
            env,
            stdout: String::new(),
            steps: 0,
            objects: HashMap::new(),
            error_variants: HashSet::from(["ParseError".to_string()]),
            variant_arity: HashMap::new(),
        }
    }

    fn check_steps(&mut self, span: Span) -> Result<(), Diagnostic> {
        self.steps += 1;
        if self.steps > 10_000_000 {
            Err(Diagnostic::new(ErrorKind::Other, "runaway loop", span))
        } else {
            Ok(())
        }
    }

    pub fn run_items(&mut self, items: &[Item]) -> Result<(), Diagnostic> {
        for item in items {
            self.run_item(item)?;
        }
        Ok(())
    }

    fn run_item(&mut self, item: &Item) -> Result<(), Diagnostic> {
        match &item.kind {
            ItemKind::Fn(f) => {
                let func = Value::Func(Rc::new(f.clone()));
                self.env.borrow_mut().define(f.name.text.clone(), func);
            }
            ItemKind::Const(c) => {
                let val = self.eval_expr(&c.value)?;
                self.env.borrow_mut().define(c.name.text.clone(), val);
            }
            ItemKind::Object(o) => {
                self.objects.insert(o.name.text.clone(), o.clone());
                self.env
                    .borrow_mut()
                    .define(o.name.text.clone(), Value::Ctor(o.name.text.clone()));
            }
            ItemKind::Error(e) => {
                for v in &e.variants {
                    self.error_variants.insert(v.text.clone());
                    self.variant_arity.insert(v.text.clone(), 0);
                    self.env
                        .borrow_mut()
                        .define(v.text.clone(), Value::Variant(v.text.clone(), vec![]));
                }
            }
            ItemKind::Enum(e) => {
                for v in &e.variants {
                    self.variant_arity.insert(v.name.text.clone(), v.payload.len());
                    self.env
                        .borrow_mut()
                        .define(v.name.text.clone(), Value::Variant(v.name.text.clone(), vec![]));
                }
            }
            ItemKind::Import(imp) => {
                Self::register_import(&self.env, imp, &mut Vec::new());
            }
            _ => {}
        }
        Ok(())
    }

    /// Bind imported leaf names to the builtins they name (`output`, `args`, ...).
    fn register_import(
        env: &Rc<RefCell<Environment>>,
        imp: &crate::ast::item::ImportDef,
        prefix: &mut Vec<String>,
    ) {
        for seg in &imp.path {
            prefix.push(seg.text.clone());
        }
        if imp.children.is_empty() {
            if let Some(leaf) = prefix.last() {
                let canonical: Option<&'static str> = match leaf.as_str() {
                    "output" => Some("output"),
                    "input" => Some("input"),
                    "args" => Some("args"),
                    "choice" => Some("choice"),
                    _ => None,
                };
                if let Some(name) = canonical {
                    env.borrow_mut().define(leaf.clone(), Value::Builtin(name));
                }
            }
        }
        for child in &imp.children {
            if let ItemKind::Import(inner) = &child.kind {
                Self::register_import(env, inner, prefix);
            }
            while prefix.len() > 1 && prefix.last().map(|s| s.as_str()) != Some("") {
                break;
            }
        }
        let _ = prefix.pop();
    }

    pub fn call_main(&mut self) -> Result<(), Diagnostic> {
        let main_val = self.env.borrow().get("main");
        if let Some(Value::Func(f)) = main_val {
            self.call_func(&f, vec![], Span::default())?;
        } else {
            return Err(Diagnostic::new(ErrorKind::Other, "main not found", Span::default()));
        }
        Ok(())
    }

    pub fn eval_block(&mut self, block: &Block) -> Result<Flow, Diagnostic> {
        let mut last = Value::Unit;
        for stmt in &block.stmts {
            let flow = self.eval_stmt(stmt)?;
            match flow {
                Flow::Normal => {
                    if let StmtKind::Expr(e) = &stmt.kind {
                        // Re-evaluating would double side effects; the value is
                        // captured through `last_value` instead.
                        let _ = e;
                    }
                    continue;
                }
                Flow::Value(v) => {
                    last = v;
                    continue;
                }
                other => return Ok(other),
            }
        }
        Ok(Flow::Value(last))
    }

    pub fn eval_stmt(&mut self, stmt: &Stmt) -> Result<Flow, Diagnostic> {
        self.check_steps(stmt.span)?;
        match &stmt.kind {
            StmtKind::Expr(e) => {
                let v = self.eval_expr(e)?;
                if let Some(r) = self.pending_return.take() {
                    return Ok(Flow::Return(r));
                }
                Ok(Flow::Value(v))
            }
            StmtKind::Assign { target, value } => {
                let val = self.eval_expr(value)?;
                match &target.kind {
                    ExprKind::Name(n) => {
                        let mut final_val = val.clone();
                        if n.text.starts_with("__it_") {
                            if let Value::List(l) = &final_val {
                                final_val = Value::Iterator(Rc::clone(l), Rc::new(RefCell::new(0)));
                            }
                        }
                        let ok = self.env.borrow_mut().set(n.text.clone(), final_val.clone());
                        if !ok {
                            self.env.borrow_mut().define(n.text.clone(), final_val);
                        }
                    }
                    ExprKind::Index { base, index } => {
                        let b = self.eval_expr(base)?;
                        let idx = self.eval_expr(index)?;
                        if let Value::List(list) = b {
                            if let Value::Int(i) = idx {
                                let mut l = list.borrow_mut();
                                if i >= 0 && (i as usize) < l.len() {
                                    l[i as usize] = val;
                                } else {
                                    return Err(Diagnostic::new(ErrorKind::Other, "index out of bounds", target.span));
                                }
                            } else {
                                return Err(Diagnostic::new(ErrorKind::Other, "index must be int", target.span));
                            }
                        } else {
                            return Err(Diagnostic::new(ErrorKind::Other, "can only index list", target.span));
                        }
                    }
                    ExprKind::Field { base, name } => {
                        let b = self.eval_expr(base)?;
                        if let Value::Object(obj) = b {
                            obj.borrow_mut().insert(name.text.clone(), val);
                        } else {
                            return Err(Diagnostic::new(ErrorKind::Other, "can only set field on object", target.span));
                        }
                    }
                    _ => return Err(Diagnostic::new(ErrorKind::Other, "invalid assignment target", target.span)),
                }
                Ok(Flow::Normal)
            }
            StmtKind::While { cond, body } => {
                loop {
                    self.check_steps(stmt.span)?;
                    let c = self.eval_expr(cond)?;
                    if let Value::Bool(b) = c {
                        if !b {
                            break;
                        }
                        match self.eval_block(body)? {
                            Flow::Normal | Flow::Value(_) | Flow::Continue => continue,
                            Flow::Break => break,
                            Flow::Return(v) => return Ok(Flow::Return(v)),
                        }
                    } else {
                        return Err(Diagnostic::new(ErrorKind::Other, format!("condition must be bool: {:?}", cond.kind), cond.span));
                    }
                }
                Ok(Flow::Normal)
            }
            StmtKind::Loop { body } => {
                loop {
                    self.check_steps(stmt.span)?;
                    match self.eval_block(body)? {
                        Flow::Normal | Flow::Value(_) | Flow::Continue => continue,
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                    }
                }
                Ok(Flow::Normal)
            }
            StmtKind::For { .. } => {
                Ok(Flow::Normal)
            }
            StmtKind::Break => Ok(Flow::Break),
            StmtKind::Continue => Ok(Flow::Continue),
            StmtKind::Return(e) => {
                if let Some(e) = e {
                    let val = self.eval_expr(e)?;
                    Ok(Flow::Return(val))
                } else {
                    Ok(Flow::Return(Value::Unit))
                }
            }
            StmtKind::Match { scrutinee, arms } => {
                let val = self.eval_expr(scrutinee)?;
                for arm in arms {
                    if let Value::Variant(ref v_name, ref args) = val {
                        if arm.pattern.name.text == *v_name {
                            let old_env = Rc::clone(&self.env);
                            let new_env = Rc::new(RefCell::new(Environment::with_parent(old_env.clone())));
                            for (i, bind) in arm.pattern.binds.iter().enumerate() {
                                if i < args.len() {
                                    new_env.borrow_mut().define(bind.text.clone(), args[i].clone());
                                }
                            }
                            self.env = new_env;
                            let flow = self.eval_stmt(&arm.body)?;
                            self.env = old_env;
                            return Ok(flow);
                        }
                    }
                    if arm.pattern.name.text == "_" {
                        let flow = self.eval_stmt(&arm.body)?;
                        return Ok(flow);
                    }
                }
                Err(Diagnostic::new(ErrorKind::Other, "no match arm", stmt.span))
            }
            StmtKind::Item(item) => {
                self.run_item(item)?;
                Ok(Flow::Normal)
            }
        }
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, Diagnostic> {
        self.check_steps(expr.span)?;
        match &expr.kind {
            ExprKind::Int(v) => Ok(Value::Int(*v)),
            ExprKind::Float(v) => Ok(Value::Float(*v)),
            ExprKind::Bool(v) => Ok(Value::Bool(*v)),
            ExprKind::Str(v) => Ok(Value::Str(v.clone())),
            ExprKind::FStr(parts) => {
                let mut res = String::new();
                for part in parts {
                    match part {
                        FPart::Text(t) => res.push_str(t),
                        FPart::Expr(e_str) => {
                            let wrapper = format!("test \"dummy\"\n  {}\n", e_str);
                            let p = crate::driver::parse_source("fstr", &wrapper);
                            if let Ok(parsed) = p {
                                if let Some(Item { kind: ItemKind::Test(test_def), .. }) = parsed.items.get(0) {
                                    if let Some(Stmt { kind: StmtKind::Expr(e), .. }) = test_def.body.stmts.get(0) {
                                        let v = self.eval_expr(e)?;
                                        res.push_str(&self.value_to_string(&v));
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(Value::Str(res))
            }
            ExprKind::Null => Ok(Value::Null),
            ExprKind::Name(n) => {
                if let Some(v) = self.env.borrow().get(&n.text) {
                    Ok(v)
                } else if n.text == "__comprehension" {
                    Ok(Value::Builtin("__comprehension"))
                } else {
                    Err(Diagnostic::new(ErrorKind::Other, format!("undefined name: {}", n.text), expr.span))
                }
            }
            ExprKind::SelfExpr => {
                if let Some(v) = self.env.borrow().get("self") {
                    Ok(v)
                } else {
                    Err(Diagnostic::new(ErrorKind::Other, "self used outside object", expr.span))
                }
            }
            ExprKind::List(items) => {
                let mut vals = vec![];
                for e in items {
                    vals.push(self.eval_expr(e)?);
                }
                Ok(Value::List(Rc::new(RefCell::new(vals))))
            }
            ExprKind::Binary { op, lhs, rhs } => {
                let l = self.eval_expr(lhs)?;
                let r = self.eval_expr(rhs)?;
                match op {
                    BinOp::Add => {
                        match (l, r) {
                            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                            (Value::List(a), Value::List(b)) => {
                                let to_add = b.borrow().clone();
                                a.borrow_mut().extend(to_add);
                                Ok(Value::Unit)
                            }
                            (Value::List(a), b) => {
                                a.borrow_mut().push(b);
                                Ok(Value::Unit)
                            }
                            _ => Err(Diagnostic::new(ErrorKind::Other, "invalid types for +", expr.span)),
                        }
                    }
                    BinOp::Sub => {
                        match (l, r) {
                            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                            _ => Err(Diagnostic::new(ErrorKind::Other, "invalid types for -", expr.span)),
                        }
                    }
                    BinOp::Mul => {
                        match (l, r) {
                            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                            _ => Err(Diagnostic::new(ErrorKind::Other, "invalid types for *", expr.span)),
                        }
                    }
                    BinOp::Div => {
                        match (l, r) {
                            (Value::Int(a), Value::Int(b)) => {
                                if b == 0 {
                                    Err(Diagnostic::new(ErrorKind::Other, "division by zero", expr.span))
                                } else {
                                    Ok(Value::Int(a / b))
                                }
                            }
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                            _ => Err(Diagnostic::new(ErrorKind::Other, "invalid types for /", expr.span)),
                        }
                    }
                    BinOp::Rem => {
                        match (l, r) {
                            (Value::Int(a), Value::Int(b)) => {
                                if b == 0 {
                                    Err(Diagnostic::new(ErrorKind::Other, "division by zero", expr.span))
                                } else {
                                    Ok(Value::Int(a % b))
                                }
                            }
                            _ => Err(Diagnostic::new(ErrorKind::Other, "invalid types for %", expr.span)),
                        }
                    }
                    BinOp::Eq => Ok(Value::Bool(l == r)),
                    BinOp::Ne => Ok(Value::Bool(l != r)),
                    BinOp::Lt => {
                        match (l, r) {
                            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
                            (l, r) => Err(Diagnostic::new(ErrorKind::Other, format!("invalid types for <: {:?} and {:?}", l, r), expr.span)),
                        }
                    }
                    BinOp::Gt => {
                        match (l, r) {
                            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
                            _ => Err(Diagnostic::new(ErrorKind::Other, "invalid types for >", expr.span)),
                        }
                    }
                    BinOp::Le => {
                        match (l, r) {
                            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
                            _ => Err(Diagnostic::new(ErrorKind::Other, "invalid types for <=", expr.span)),
                        }
                    }
                    BinOp::Ge => {
                        match (l, r) {
                            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
                            _ => Err(Diagnostic::new(ErrorKind::Other, "invalid types for >=", expr.span)),
                        }
                    }
                    BinOp::And => {
                        if let Value::Bool(a) = l {
                            if !a {
                                Ok(Value::Bool(false))
                            } else {
                                if let Value::Bool(b) = r {
                                    Ok(Value::Bool(b))
                                } else {
                                    Err(Diagnostic::new(ErrorKind::Other, "invalid types for and", expr.span))
                                }
                            }
                        } else {
                            Err(Diagnostic::new(ErrorKind::Other, "invalid types for and", expr.span))
                        }
                    }
                    BinOp::Or => {
                        if let Value::Bool(a) = l {
                            if a {
                                Ok(Value::Bool(true))
                            } else {
                                if let Value::Bool(b) = r {
                                    Ok(Value::Bool(b))
                                } else {
                                    Err(Diagnostic::new(ErrorKind::Other, "invalid types for or", expr.span))
                                }
                            }
                        } else {
                            Err(Diagnostic::new(ErrorKind::Other, "invalid types for or", expr.span))
                        }
                    }
                }
            }
            ExprKind::Unary { op, operand } => {
                let v = self.eval_expr(operand)?;
                match op {
                    UnOp::Not => {
                        if let Value::Bool(b) = v {
                            Ok(Value::Bool(!b))
                        } else {
                            Err(Diagnostic::new(ErrorKind::Other, "invalid type for not", expr.span))
                        }
                    }
                    UnOp::Neg => {
                        match v {
                            Value::Int(i) => Ok(Value::Int(-i)),
                            Value::Float(f) => Ok(Value::Float(-f)),
                            _ => Err(Diagnostic::new(ErrorKind::Other, "invalid type for neg", expr.span)),
                        }
                    }
                }
            }
            ExprKind::Call { callee, args } => {
                // Object construction needs argument names: `Circle(radius = 2.0)`.
                if let ExprKind::Name(n) = &callee.kind {
                    if matches!(self.env.borrow().get(&n.text), Some(Value::Ctor(_))) {
                        let mut named = Vec::new();
                        for arg in args {
                            let v = self.eval_expr(&arg.value)?;
                            named.push((arg.name.as_ref().map(|x| x.text.clone()), v));
                        }
                        return self.construct(&n.text, named, expr.span);
                    }
                }
                let func = self.eval_expr(callee)?;
                let mut eval_args = vec![];
                for arg in args {
                    eval_args.push(self.eval_expr(&arg.value)?);
                }
                let res = self.call_value(&func, eval_args, expr.span);
                res
            }
            ExprKind::Field { base, name } => {
                let b = self.eval_expr(base)?;
                if let Value::Object(obj) = &b {
                    if let Some(v) = obj.borrow().get(&name.text) {
                        return Ok(v.clone());
                    }
                }

                if let Value::Iterator(list, cursor) = &b {
                    if name.text == "has_next" || name.text == "next" {
                        return Ok(Value::Closure(vec![], Box::new(Expr { kind: ExprKind::Null, span: expr.span }), Rc::new(RefCell::new({
                            let mut env = Environment::new();
                            env.define("self_iter_list".to_string(), Value::List(Rc::clone(list)));
                            env.define("self_iter_cursor".to_string(), Value::Iterator(Rc::clone(list), Rc::clone(cursor)));
                            env.define("self_iter_op".to_string(), Value::Str(name.text.clone()));
                            env
                        }))));
                    }
                }

                // `Stack.new(...)` — `.new` on a type produces the constructor.
                if let Value::Ctor(ty) = &b {
                    // A user-declared `new` method wins over the implicit constructor.
                    if let Some(def) = self.objects.get(ty) {
                        if let Some(m) = def.methods.iter().find(|m| m.name.text == name.text) {
                            let mut map = std::collections::BTreeMap::new();
                            map.insert("__type".to_string(), Value::Str(ty.clone()));
                            return Ok(Value::BoundMethod(
                                Rc::new(RefCell::new(map)),
                                Rc::new(m.clone()),
                            ));
                        }
                    }
                    if name.text == "new" {
                        return Ok(Value::Ctor(ty.clone()));
                    }
                }

                // Method dispatch: `c.area` on an object that declares `area`.
                if let Value::Object(obj) = &b {
                    let ty = obj.borrow().get("__type").and_then(|v| match v {
                        Value::Str(s) => Some(s.clone()),
                        _ => None,
                    });
                    if let Some(ty) = ty {
                        if let Some(def) = self.objects.get(&ty) {
                            if let Some(m) = def.methods.iter().find(|m| m.name.text == name.text) {
                                return Ok(Value::BoundMethod(Rc::clone(obj), Rc::new(m.clone())));
                            }
                        }
                    }
                }

                match (&b, name.text.as_str()) {
                    (Value::Str(s), "len") => {
                        Ok(Value::Closure(vec![], Box::new(Expr { kind: ExprKind::Null, span: expr.span }), Rc::new(RefCell::new({
                            let mut env = Environment::new();
                            env.define("self_str".to_string(), Value::Str(s.clone()));
                            env.define("self_str_op".to_string(), Value::Str("len".to_string()));
                            env
                        }))))
                    }
                    (Value::Str(s), "parse") => {
                        let txt = s.clone();
                        Ok(Value::Closure(vec![], Box::new(Expr { kind: ExprKind::Null, span: expr.span }), Rc::new(RefCell::new({
                            let mut e = Environment::new();
                            e.define("self_parse".to_string(), Value::Str(txt));
                            e
                        }))))
                    }
                    (Value::Str(s), "split") => {
                        let txt = s.clone();
                        Ok(Value::Closure(vec!["sep".to_string()], Box::new(Expr { kind: ExprKind::Null, span: expr.span }), Rc::new(RefCell::new({
                            let mut e = Environment::new();
                            e.define("self_split".to_string(), Value::Str(txt));
                            e
                        }))))
                    }
                    (Value::Str(s), "contains") => {
                        Ok(Value::Closure(vec!["sub".to_string()], Box::new(Expr { kind: ExprKind::Null, span: expr.span }), Rc::new(RefCell::new({
                            let mut env = Environment::new();
                            env.define("self_str".to_string(), Value::Str(s.clone()));
                            env.define("self_str_op".to_string(), Value::Str("contains".to_string()));
                            env
                        }))))
                    }
                    (Value::Str(s), "lower") => {
                        Ok(Value::Closure(vec![], Box::new(Expr { kind: ExprKind::Null, span: expr.span }), Rc::new(RefCell::new({
                            let mut env = Environment::new();
                            env.define("self_str".to_string(), Value::Str(s.clone()));
                            env.define("self_str_op".to_string(), Value::Str("lower".to_string()));
                            env
                        }))))
                    }
                    (Value::List(l), "sort") => {
                        Ok(Value::Closure(vec![], Box::new(Expr { kind: ExprKind::Null, span: expr.span }), Rc::new(RefCell::new({
                            let mut e = Environment::new();
                            e.define("self_list".to_string(), Value::List(Rc::clone(l)));
                            e.define("self_list_op".to_string(), Value::Str("sort".to_string()));
                            e
                        }))))
                    }
                    (Value::List(l), "reduce") => {
                        Ok(Value::Closure(vec!["init".to_string(), "f".to_string()], Box::new(Expr { kind: ExprKind::Null, span: expr.span }), Rc::new(RefCell::new({
                            let mut e = Environment::new();
                            e.define("self_list".to_string(), Value::List(Rc::clone(l)));
                            e.define("self_list_op".to_string(), Value::Str("reduce".to_string()));
                            e
                        }))))
                    }
                    (Value::List(l), "filter") => {
                        Ok(Value::Closure(vec!["f".to_string()], Box::new(Expr { kind: ExprKind::Null, span: expr.span }), Rc::new(RefCell::new({
                            let mut e = Environment::new();
                            e.define("self_list".to_string(), Value::List(Rc::clone(l)));
                            e.define("self_list_op".to_string(), Value::Str("filter".to_string()));
                            e
                        }))))
                    }
                    (Value::List(l), "map") => {
                        Ok(Value::Closure(vec!["f".to_string()], Box::new(Expr { kind: ExprKind::Null, span: expr.span }), Rc::new(RefCell::new({
                            let mut e = Environment::new();
                            e.define("self_list".to_string(), Value::List(Rc::clone(l)));
                            e.define("self_list_op".to_string(), Value::Str("map".to_string()));
                            e
                        }))))
                    }
                    (Value::List(l), "len") => {
                        Ok(Value::Closure(vec![], Box::new(Expr { kind: ExprKind::Null, span: expr.span }), Rc::new(RefCell::new({
                            let mut env = Environment::new();
                            env.define("self_list".to_string(), Value::List(Rc::clone(l)));
                            env.define("self_list_op".to_string(), Value::Str("len".to_string()));
                            env
                        }))))
                    }
                    (Value::List(l), "add") => {
                        Ok(Value::Closure(vec!["item".to_string()], Box::new(Expr { kind: ExprKind::Null, span: expr.span }), Rc::new(RefCell::new({
                            let mut env = Environment::new();
                            env.define("self_list".to_string(), Value::List(Rc::clone(l)));
                            env.define("self_list_op".to_string(), Value::Str("add".to_string()));
                            env
                        }))))
                    }
                    (Value::List(l), "extend") => {
                        Ok(Value::Closure(vec!["other".to_string()], Box::new(Expr { kind: ExprKind::Null, span: expr.span }), Rc::new(RefCell::new({
                            let mut env = Environment::new();
                            env.define("self_list".to_string(), Value::List(Rc::clone(l)));
                            env.define("self_list_op".to_string(), Value::Str("extend".to_string()));
                            env
                        }))))
                    }
                    _ => Err(Diagnostic::new(ErrorKind::Other, format!("no field {}", name.text), expr.span))
                }
            }
            ExprKind::Index { base, index } => {
                let b = self.eval_expr(base)?;
                // `T[int]` is a type application, not a list index.
                if matches!(
                    b,
                    Value::Ctor(_)
                        | Value::Func(_)
                        | Value::Builtin(_)
                        | Value::Closure(..)
                        | Value::BoundMethod(..)
                ) && matches!(index.kind, ExprKind::Name(_))
                {
                    return Ok(b);
                }
                let i = self.eval_expr(index)?;
                if let Value::List(l) = b {
                    if let Value::Int(idx) = i {
                        if idx >= 0 && (idx as usize) < l.borrow().len() {
                            Ok(l.borrow()[idx as usize].clone())
                        } else {
                            Err(Diagnostic::new(ErrorKind::Other, "index out of bounds", expr.span))
                        }
                    } else {
                        Err(Diagnostic::new(ErrorKind::Other, "index must be int", expr.span))
                    }
                } else {
                    Err(Diagnostic::new(ErrorKind::Other, "can only index list", expr.span))
                }
            }
            ExprKind::Lambda { params, body } => {
                let param_names: Vec<String> = params.iter().map(|n| n.text.clone()).collect();
                Ok(Value::Closure(param_names, body.clone(), Rc::clone(&self.env)))
            }
            ExprKind::If { cond, then, elifs, else_ } => {
                let flow = self.eval_if_flow(cond, then, elifs, else_.as_ref(), expr.span)?;
                match flow {
                    Flow::Return(v) => {
                        self.pending_return = Some(v);
                        Ok(Value::Unit)
                    }
                    Flow::Value(v) => Ok(v),
                    _ => Ok(Value::Unit),
                }
            }
            ExprKind::Comprehension { .. } => Err(Diagnostic::new(ErrorKind::Other, "unlowered comprehension", expr.span)),
            ExprKind::Range { from, to } => {
                let start = match from {
                    Some(e) => match self.eval_expr(e)? {
                        Value::Int(i) => i,
                        _ => return Err(Diagnostic::new(ErrorKind::Other, "range bounds must be int", expr.span)),
                    },
                    None => 1,
                };
                let end = match to {
                    Some(e) => match self.eval_expr(e)? {
                        Value::Int(i) => i,
                        _ => return Err(Diagnostic::new(ErrorKind::Other, "range bounds must be int", expr.span)),
                    },
                    None => return Err(Diagnostic::new(ErrorKind::Other, "open range needs an end", expr.span)),
                };
                let mut out = Vec::new();
                let mut i = start;
                while i < end {
                    out.push(Value::Int(i));
                    i += 1;
                }
                Ok(Value::List(Rc::new(RefCell::new(out))))
            }
            ExprKind::Propagate(_) => Err(Diagnostic::new(ErrorKind::Other, "unsupported propagate", expr.span)),
        }
    }

    /// Evaluate an if/elif/else chain, preserving `return` and the value of the
    /// chosen branch (so `x = if c => a else b` works after desugaring).
    fn eval_if_flow(
        &mut self,
        cond: &Expr,
        then: &Block,
        elifs: &[(Expr, Block)],
        else_: Option<&Block>,
        _span: Span,
    ) -> Result<Flow, Diagnostic> {
        match self.eval_expr(cond)? {
            Value::Bool(true) => return self.eval_block(then),
            Value::Bool(false) => {}
            _ => {
                return Err(Diagnostic::new(
                    ErrorKind::Other,
                    "if condition must be bool",
                    cond.span,
                ))
            }
        }
        for (elif_cond, elif_then) in elifs {
            match self.eval_expr(elif_cond)? {
                Value::Bool(true) => return self.eval_block(elif_then),
                Value::Bool(false) => {}
                _ => {
                    return Err(Diagnostic::new(
                        ErrorKind::Other,
                        "elif condition must be bool",
                        elif_cond.span,
                    ))
                }
            }
        }
        match else_ {
            Some(b) => self.eval_block(b),
            None => Ok(Flow::Normal),
        }
    }

    /// Build an object from `ObjectDef`, mapping named args to fields by name.
    fn construct(
        &mut self,
        name: &str,
        args: Vec<(Option<String>, Value)>,
        span: Span,
    ) -> Result<Value, Diagnostic> {
        let def = match self.objects.get(name) {
            Some(d) => d.clone(),
            None => {
                return Err(Diagnostic::new(
                    ErrorKind::Other,
                    format!("unknown type `{}`", name),
                    span,
                ))
            }
        };
        let mut map = std::collections::BTreeMap::new();
        map.insert("__type".to_string(), Value::Str(name.to_string()));
        let mut positional = args.iter().filter(|(n, _)| n.is_none()).map(|(_, v)| v.clone());
        for field in &def.fields {
            let named = args
                .iter()
                .find(|(n, _)| n.as_deref() == Some(field.name.text.as_str()))
                .map(|(_, v)| v.clone());
            let val = match named {
                Some(v) => v,
                None => positional.next().unwrap_or(Value::Null),
            };
            map.insert(field.name.text.clone(), val);
        }
        Ok(Value::Object(Rc::new(RefCell::new(map))))
    }

    fn call_value(&mut self, callee: &Value, args: Vec<Value>, span: Span) -> Result<Value, Diagnostic> {
        match callee {
            Value::Func(f) => self.call_func(f, args, span),
            Value::Ctor(name) => {
                let named = args.into_iter().map(|v| (None, v)).collect();
                self.construct(name, named, span)
            }
            Value::Variant(name, existing) => {
                let mut all = existing.clone();
                all.extend(args);
                Ok(Value::Variant(name.clone(), all))
            }
            Value::BoundMethod(obj, f) => {
                let mut call_env = Environment::with_parent(Rc::clone(&self.env));
                call_env.define("self".to_string(), Value::Object(Rc::clone(obj)));
                if let Some(Value::Str(ty)) = obj.borrow().get("__type").cloned() {
                    call_env.define("Self".to_string(), Value::Ctor(ty));
                }
                for (i, p) in f.params.iter().enumerate() {
                    if i < args.len() {
                        call_env.define(p.name.text.clone(), args[i].clone());
                    }
                }
                let old_env = Rc::clone(&self.env);
                self.env = Rc::new(RefCell::new(call_env));
                let flow = self.eval_block(&f.body);
                self.env = old_env;
                match flow? {
                    Flow::Return(v) => Ok(v),
                    _ => Ok(Value::Unit),
                }
            }
            Value::Closure(params, body, env) => {
                if body.kind == ExprKind::Null {
                    let closure_env = env.borrow();
                    if let Some(Value::Str(txt)) = closure_env.get("self_parse") {
                        return match txt.trim().parse::<i64>() {
                            Ok(i) => Ok(Value::Int(i)),
                            Err(_) => match txt.trim().parse::<f64>() {
                                Ok(f) => Ok(Value::Float(f)),
                                Err(_) => Ok(Value::Variant("ParseError".to_string(), vec![])),
                            },
                        };
                    }
                    if let Some(Value::Str(txt)) = closure_env.get("self_split") {
                        let sep = match args.first() {
                            Some(Value::Str(s)) => s.clone(),
                            _ => " ".to_string(),
                        };
                        let parts: Vec<Value> =
                            txt.split(sep.as_str()).map(|p| Value::Str(p.to_string())).collect();
                        return Ok(Value::List(Rc::new(RefCell::new(parts))));
                    }
                    if let Some(Value::Str(s)) = closure_env.get("self_str") {
                        if let Some(Value::Str(op)) = closure_env.get("self_str_op") {
                            if op == "len" {
                                return Ok(Value::Int(s.len() as i64));
                            } else if op == "contains" {
                                if let Value::Str(sub) = &args[0] {
                                    return Ok(Value::Bool(s.contains(sub)));
                                }
                            } else if op == "lower" {
                                return Ok(Value::Str(s.to_lowercase()));
                            }
                        }
                    }
                    if let Some(Value::Iterator(list, cursor)) = closure_env.get("self_iter_cursor") {
                        if let Some(Value::Str(op)) = closure_env.get("self_iter_op") {
                            if op == "has_next" {
                                let len = list.borrow().len();
                                let cur = *cursor.borrow();
                                return Ok(Value::Bool(cur < len));
                            } else if op == "next" {
                                let mut c = cursor.borrow_mut();
                                let len = list.borrow().len();
                                if *c < len {
                                    let val = list.borrow()[*c].clone();
                                    *c += 1;
                                    return Ok(val);
                                } else {
                                    return Err(Diagnostic::new(ErrorKind::Other, "iterator exhausted", span));
                                }
                            }
                        }
                    }
                    if let Some(Value::List(l)) = closure_env.get("self_list") {
                        if let Some(Value::Str(op)) = closure_env.get("self_list_op") {
                            if op == "len" {
                                return Ok(Value::Int(l.borrow().len() as i64));
                            } else if op == "add" {
                                l.borrow_mut().push(args[0].clone());
                                return Ok(Value::Unit);
                            } else if op == "reduce" {
                                let init = args.first().cloned().unwrap_or(Value::Null);
                                let f = args.get(1).cloned().unwrap_or(Value::Null);
                                let items = l.borrow().clone();
                                let mut acc = init;
                                for it in items {
                                    acc = self.call_value(&f, vec![acc, it], span)?;
                                }
                                return Ok(acc);
                            } else if op == "sort" {
                                l.borrow_mut().sort_by(|a, b| match (a, b) {
                                    (Value::Int(x), Value::Int(y)) => x.cmp(y),
                                    (Value::Str(x), Value::Str(y)) => x.cmp(y),
                                    _ => std::cmp::Ordering::Equal,
                                });
                                return Ok(Value::Unit);
                            } else if op == "extend" {
                                if let Value::List(other) = &args[0] {
                                    let to_add = other.borrow().clone();
                                    l.borrow_mut().extend(to_add);
                                }
                                return Ok(Value::Unit);
                            }
                        }
                    }
                }

                let mut call_env = Environment::with_parent(Rc::clone(env));
                for (i, param) in params.iter().enumerate() {
                    if i < args.len() {
                        call_env.define(param.clone(), args[i].clone());
                    }
                }
                let old_env = Rc::clone(&self.env);
                self.env = Rc::new(RefCell::new(call_env));
                let res = self.eval_expr(body)?;
                self.env = old_env;
                Ok(res)
            }
            Value::Builtin(name) => {
                match *name {
                    "output" => {
                        if args.len() == 2 {
                            if let Value::Builtin("f") = args[0] {
                                if let Value::Str(s) = &args[1] {
                                    let mut res = String::new();
                                    let mut in_expr = false;
                                    let mut current_expr = String::new();
                                    for c in s.chars() {
                                        if c == '{' {
                                            in_expr = true;
                                            current_expr.clear();
                                        } else if c == '}' {
                                            in_expr = false;
                                            let val = self.env.borrow().get(&current_expr).unwrap_or(Value::Null);
                                            res.push_str(&self.value_to_string(&val));
                                        } else {
                                            if in_expr {
                                                current_expr.push(c);
                                            } else {
                                                res.push(c);
                                            }
                                        }
                                    }
                                    self.stdout.push_str(&res);
                                    self.stdout.push('\n');
                                    return Ok(Value::Unit);
                                }
                            }
                        }
                        let s = args.iter().map(|a| self.value_to_string(a)).collect::<Vec<_>>().join(" ");
                        self.stdout.push_str(&s);
                        self.stdout.push('\n');
                        Ok(Value::Unit)
                    }
                    "input" => {
                        Err(Diagnostic::new(ErrorKind::Other, "input is an error", span))
                    }
                    "choice" => {
                        let items = match args.first() {
                            Some(Value::List(l)) => l.borrow().clone(),
                            _ => Vec::new(),
                        };
                        if items.is_empty() {
                            return Ok(Value::Null);
                        }
                        // Deterministic pick so runs are reproducible.
                        let n = self.steps.wrapping_mul(6364136223846793005).wrapping_add(1);
                        Ok(items[(n as usize >> 16) % items.len()].clone())
                    }
                    "args" => Ok(Value::List(Rc::new(RefCell::new(
                        std::env::args().skip(1).map(Value::Str).collect(),
                    )))),
                    "is_error" => {
                        let v = args.first().cloned().unwrap_or(Value::Null);
                        let is = match &v {
                            Value::Variant(n, _) => self.error_variants.contains(n),
                            _ => false,
                        };
                        Ok(Value::Bool(is))
                    }
                    "f" => {
                        if let Some(Value::Str(s)) = args.get(0) {
                            let mut res = String::new();
                            let mut in_expr = false;
                            let mut current_expr = String::new();
                            for c in s.chars() {
                                if c == '{' {
                                    in_expr = true;
                                    current_expr.clear();
                                } else if c == '}' {
                                    in_expr = false;
                                    let val = self.env.borrow().get(&current_expr).unwrap_or(Value::Null);
                                    res.push_str(&self.value_to_string(&val));
                                } else {
                                    if in_expr {
                                        current_expr.push(c);
                                    } else {
                                        res.push(c);
                                    }
                                }
                            }
                            return Ok(Value::Str(res));
                        } else {
                            return Err(Diagnostic::new(ErrorKind::Other, "f expects string", span));
                        }
                    }
                    "__comprehension" => {
                        let f = &args[0];
                        let it = &args[1];
                        if let Value::List(l) = it {
                            let mut res = vec![];
                            let list = l.borrow().clone();
                            for item in list {
                                res.push(self.call_value(f, vec![item], span)?);
                            }
                            Ok(Value::List(Rc::new(RefCell::new(res))))
                        } else {
                            Err(Diagnostic::new(ErrorKind::Other, "comprehension expects list", span))
                        }
                    }
                    _ => Err(Diagnostic::new(ErrorKind::Other, format!("unknown builtin: {}", name), span)),
                }
            }
            _ => {
                Err(Diagnostic::new(ErrorKind::Other, "not callable", span))
            }
        }
    }

    fn call_func(&mut self, f: &FnDef, args: Vec<Value>, _span: Span) -> Result<Value, Diagnostic> {
        let mut call_env = Environment::with_parent(Rc::clone(&self.env));
        for (i, param) in f.params.iter().enumerate() {
            if i < args.len() {
                call_env.define(param.name.text.clone(), args[i].clone());
            }
        }
        let old_env = Rc::clone(&self.env);
        self.env = Rc::new(RefCell::new(call_env));
        let flow = self.eval_block(&f.body)?;
        self.env = old_env;
        match flow {
            Flow::Return(v) | Flow::Value(v) => Ok(v),
            _ => Ok(Value::Unit),
        }
    }

    fn value_to_string(&self, val: &Value) -> String {
        match val {
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Str(s) => s.clone(),
            Value::Null => "null".to_string(),
            Value::Unit => "()".to_string(),
            Value::List(l) => {
                let inner = l.borrow().iter().map(|v| self.value_to_string(v)).collect::<Vec<_>>().join(" ");
                format!("[{}]", inner)
            }
            Value::Object(_) => "object".to_string(),
            Value::Variant(name, _) => name.clone(),
            Value::Func(f) => format!("<fn {}>", f.name.text),
            Value::Closure(..) => "<closure>".to_string(),
            Value::Builtin(n) => format!("<builtin {}>", n),
            Value::Ctor(n) => format!("<type {}>", n),
            Value::BoundMethod(..) => "<method>".to_string(),
            Value::Iterator(..) => "<iterator>".to_string(),
        }
    }
}
