use std::collections::BTreeMap;
use crate::ast::item::FnDef;
use crate::ast::expr::Expr;
use std::rc::Rc;
use std::cell::RefCell;
use crate::interp::eval::Environment;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Null,
    Unit,
    List(Rc<RefCell<Vec<Value>>>),
    Object(Rc<RefCell<BTreeMap<String, Value>>>),
    Variant(String, Vec<Value>),
    Func(Rc<FnDef>),
    Closure(Vec<String>, Box<Expr>, Rc<RefCell<Environment>>),
    Builtin(&'static str),
    Iterator(Rc<RefCell<Vec<Value>>>, Rc<RefCell<usize>>),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::Str(_) => "string",
            Value::Null => "null",
            Value::Unit => "!",
            Value::List(_) => "list",
            Value::Object(_) => "object",
            Value::Variant(_, _) => "variant",
            Value::Iterator(..) => "iterator",
            Value::Func(_) | Value::Closure(..) | Value::Builtin(_) => "fn",
        }
    }
}
