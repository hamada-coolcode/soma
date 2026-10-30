use std::collections::HashMap;

use crate::ast::expr::ExprKind;
use crate::ast::ty::{TypeKind, TypeRepr};

pub type TypeId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    Str,
    Null,
    Never,
    Unit,
    Named { name: String, args: Vec<TypeId> },
    List(TypeId),
    Array(TypeId, usize),
    Optional(TypeId),
    Fn { params: Vec<TypeId>, ret: TypeId },
    ErrorUnion { err: TypeId, ok: TypeId },
    Var(u32),
}

#[derive(Debug, Clone, Default)]
pub struct TypeTable {
    types: Vec<Type>,
    vars: Vec<Option<Type>>,
}

impl TypeTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(&mut self, ty: Type) -> TypeId {
        self.types.push(ty);
        self.types.len() - 1
    }

    pub fn ty(&self, id: TypeId) -> &Type {
        &self.types[id]
    }

    pub fn resolve(&self, id: TypeId) -> Type {
        let mut curr = self.types[id].clone();
        loop {
            let next = match curr {
                Type::Var(v) => {
                    if let Some(bound) = self.var(v) {
                        bound.clone()
                    } else {
                        break curr;
                    }
                }
                _ => break curr,
            };
            curr = next;
        }
    }

    pub fn fresh_var(&mut self) -> TypeId {
        let n = self.vars.len() as u32;
        self.vars.push(None);
        self.intern(Type::Var(n))
    }

    pub fn var(&self, v: u32) -> Option<&Type> {
        self.vars[v as usize].as_ref()
    }

    pub fn set_var(&mut self, v: u32, ty: Type) {
        self.vars[v as usize] = Some(ty);
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    pub fn display(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "int".to_string(),
            Type::Float => "float".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Str => "string".to_string(),
            Type::Null => "null".to_string(),
            Type::Never => "!".to_string(),
            Type::Unit => "()".to_string(),
            Type::Named { name, args } => {
                if args.is_empty() {
                    name.clone()
                } else {
                    let arg_strs: Vec<String> = args
                        .iter()
                        .map(|&id| self.display(&self.resolve(id)))
                        .collect();
                    format!("{}[{}]", name, arg_strs.join(", "))
                }
            }
            Type::List(elem) => format!("{}[]", self.display(&self.resolve(*elem))),
            Type::Array(elem, size) => format!("{}[{}]", self.display(&self.resolve(*elem)), size),
            Type::Optional(elem) => format!("{}?", self.display(&self.resolve(*elem))),
            Type::Fn { params, ret } => {
                let param_strs: Vec<String> = params
                    .iter()
                    .map(|&id| self.display(&self.resolve(id)))
                    .collect();
                format!("({}) {}", param_strs.join(", "), self.display(&self.resolve(*ret)))
            }
            Type::ErrorUnion { err, ok } => {
                format!("{}!{}", self.display(&self.resolve(*err)), self.display(&self.resolve(*ok)))
            }
            Type::Var(_) => "_".to_string(),
        }
    }

    pub fn display_id(&self, id: TypeId) -> String {
        self.display(&self.resolve(id))
    }

    pub fn from_repr(&mut self, repr: &TypeRepr, env: &HashMap<String, u32>) -> TypeId {
        match &repr.kind {
            TypeKind::Named { name, generics } => {
                let name_str = name.text.clone();
                if generics.is_empty() {
                    if let Some(&var_id) = env.get(&name_str) {
                        return self.intern(Type::Var(var_id));
                    }
                    match name_str.as_str() {
                        "int" => return self.intern(Type::Int),
                        "float" => return self.intern(Type::Float),
                        "bool" => return self.intern(Type::Bool),
                        "string" => return self.intern(Type::Str),
                        "null" => return self.intern(Type::Null),
                        "()" => return self.intern(Type::Unit),
                        _ => {}
                    }
                }
                let mut args = Vec::new();
                for g in generics {
                    args.push(self.from_repr(g, env));
                }
                self.intern(Type::Named {
                    name: name_str,
                    args,
                })
            }
            TypeKind::Array { elem, size } => {
                let elem_id = self.from_repr(elem, env);
                if let Some(s) = size {
                    if let ExprKind::Int(v) = s.kind {
                        self.intern(Type::Array(elem_id, v as usize))
                    } else {
                        self.intern(Type::Array(elem_id, 0))
                    }
                } else {
                    self.intern(Type::List(elem_id))
                }
            }
            TypeKind::Optional(elem) => {
                let elem_id = self.from_repr(elem, env);
                self.intern(Type::Optional(elem_id))
            }
            TypeKind::Fn { params, ret } => {
                let param_ids = params.iter().map(|p| self.from_repr(p, env)).collect();
                let ret_id = self.from_repr(ret, env);
                self.intern(Type::Fn { params: param_ids, ret: ret_id })
            }
            TypeKind::ErrorUnion { err, ok } => {
                let err_id = self.from_repr(err, env);
                let ok_id = self.from_repr(ok, env);
                self.intern(Type::ErrorUnion { err: err_id, ok: ok_id })
            }
            TypeKind::Never => self.intern(Type::Never),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_display() {
        let mut table = TypeTable::new();
        let t_int = table.intern(Type::Int);
        let t_list = table.intern(Type::List(t_int));
        assert_eq!(table.display_id(t_list), "int[]");

        let t_opt = table.intern(Type::Optional(t_int));
        assert_eq!(table.display_id(t_opt), "int?");

        let t_err = table.intern(Type::ErrorUnion { err: t_int, ok: t_opt });
        assert_eq!(table.display_id(t_err), "int!int?");

        let t_fn = table.intern(Type::Fn { params: vec![t_int], ret: t_list });
        assert_eq!(table.display_id(t_fn), "(int) int[]");
    }

    #[test]
    fn test_resolve_idempotence() {
        let mut table = TypeTable::new();
        let v1 = table.fresh_var();
        let v2 = table.fresh_var();
        let _t_int = table.intern(Type::Int);

        let v1_id = match table.ty(v1) { Type::Var(id) => *id, _ => unreachable!() };
        let v2_id = match table.ty(v2) { Type::Var(id) => *id, _ => unreachable!() };

        table.set_var(v2_id, Type::Int);
        table.set_var(v1_id, Type::Var(v2_id));

        assert_eq!(table.resolve(v1), Type::Int);
        assert_eq!(table.resolve(v1), Type::Int); // Idempotent
    }
}
