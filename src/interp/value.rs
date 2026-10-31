use std::collections::BTreeMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::ast::item::FnDef;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Null,
    Unit,
    List(Rc<RefCell<Vec<Value>>>),
    Object(Rc<RefCell<BTreeMap<String, Value>>>),
    Fn(FnDef),
    BuiltinFn(String),
    Enum(String, Vec<Value>),
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
            Value::Fn(_) => "fn",
            Value::BuiltinFn(_) => "fn",
            Value::Enum(_, _) => "enum",
        }
    }
}
