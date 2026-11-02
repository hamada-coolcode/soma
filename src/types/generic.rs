use std::collections::HashMap;

use crate::ast::expr::{Expr, ExprKind};
use crate::ast::ty::{TypeKind, TypeRepr};
use crate::diag::{Diagnostic, ErrorKind};
use crate::types::ty::{Type, TypeId, TypeTable};

/// Generic instantiation and monomorphization prep (owned by task 13).
pub fn check_generics() -> Vec<Diagnostic> {
    Vec::new()
}

pub struct GenericEnv<'a> {
    pub type_params: &'a [String],
    pub known_arities: &'a HashMap<String, usize>,
}

pub fn check_type_repr(
    table: &mut TypeTable,
    repr: &TypeRepr,
    env: &GenericEnv,
    diags: &mut Vec<Diagnostic>,
) -> TypeId {
    match &repr.kind {
        TypeKind::Named { name, generics } => {
            let name_str = &name.text;

            // Is it a type parameter?
            if let Some(pos) = env.type_params.iter().position(|p| p == name_str) {
                if !generics.is_empty() {
                    diags.push(Diagnostic::new(
                        ErrorKind::Type,
                        format!("E0401: type parameter `{}` cannot take type arguments", name_str),
                        repr.span.clone(),
                    ));
                    return table.fresh_var();
                }
                return table.intern(Type::Var(pos as u32));
            }

            // Is it a known type?
            if let Some(&arity) = env.known_arities.get(name_str) {
                if generics.len() != arity {
                    diags.push(Diagnostic::new(
                        ErrorKind::Type,
                        format!(
                            "E0402: type `{}` expects {} arguments, found {}",
                            name_str,
                            arity,
                            generics.len()
                        ),
                        repr.span.clone(),
                    ));
                }

                let mut arg_ids = Vec::new();
                for g in generics {
                    arg_ids.push(check_type_repr(table, g, env, diags));
                }
                return table.intern(Type::Named {
                    name: name_str.clone(),
                    args: arg_ids,
                });
            }

            // Builtin types
            if generics.is_empty() {
                match name_str.as_str() {
                    "int" => return table.intern(Type::Int),
                    "float" => return table.intern(Type::Float),
                    "bool" => return table.intern(Type::Bool),
                    "string" => return table.intern(Type::Str),
                    "null" => return table.intern(Type::Null),
                    "uint" => return table.intern(Type::Named { name: "uint".to_string(), args: vec![] }), // sometimes used
                    "()" => return table.intern(Type::Unit),
                    _ => {}
                }
            } else {
                match name_str.as_str() {
                    "int" | "float" | "bool" | "string" | "null" | "uint" | "()" => {
                        diags.push(Diagnostic::new(
                            ErrorKind::Type,
                            format!("E0403: built-in type `{}` is not generic", name_str),
                            repr.span.clone(),
                        ));
                        return table.fresh_var();
                    }
                    _ => {}
                }
            }

            diags.push(Diagnostic::new(
                ErrorKind::Type,
                format!("E0400: undeclared type parameter or type `{}`", name_str),
                repr.span.clone(),
            ));
            table.fresh_var()
        }
        TypeKind::Array { elem, size } => {
            let elem_id = check_type_repr(table, elem, env, diags);
            if let Some(s) = size {
                if let ExprKind::Int(v) = s.kind {
                    if v >= 0 {
                        return table.intern(Type::Array(elem_id, v as usize));
                    }
                }
                diags.push(Diagnostic::new(
                    ErrorKind::Type,
                    "E0404: array size must be a non-negative integer literal",
                    s.span.clone(),
                ));
                table.intern(Type::Array(elem_id, 0))
            } else {
                table.intern(Type::List(elem_id))
            }
        }
        TypeKind::Optional(elem) => {
            let elem_id = check_type_repr(table, elem, env, diags);
            table.intern(Type::Optional(elem_id))
        }
        TypeKind::Fn { params, ret } => {
            let param_ids = params
                .iter()
                .map(|p| check_type_repr(table, p, env, diags))
                .collect();
            let ret_id = check_type_repr(table, ret, env, diags);
            table.intern(Type::Fn {
                params: param_ids,
                ret: ret_id,
            })
        }
        TypeKind::ErrorUnion { err, ok } => {
            let err_id = check_type_repr(table, err, env, diags);
            let ok_id = check_type_repr(table, ok, env, diags);
            table.intern(Type::ErrorUnion {
                err: err_id,
                ok: ok_id,
            })
        }
        TypeKind::Never => table.intern(Type::Never),
    }
}

pub fn substitute(table: &mut TypeTable, ty_id: TypeId, args: &[TypeId]) -> TypeId {
    let ty = table.resolve(ty_id);
    match ty {
        Type::Var(v) => {
            let v_usize = v as usize;
            if v_usize < args.len() {
                args[v_usize]
            } else {
                ty_id
            }
        }
        Type::List(elem) => {
            let new_elem = substitute(table, elem, args);
            if new_elem == elem {
                ty_id
            } else {
                table.intern(Type::List(new_elem))
            }
        }
        Type::Array(elem, size) => {
            let new_elem = substitute(table, elem, args);
            if new_elem == elem {
                ty_id
            } else {
                table.intern(Type::Array(new_elem, size))
            }
        }
        Type::Optional(elem) => {
            let new_elem = substitute(table, elem, args);
            if new_elem == elem {
                ty_id
            } else {
                table.intern(Type::Optional(new_elem))
            }
        }
        Type::Fn { params, ret } => {
            let new_params: Vec<_> = params.iter().map(|&p| substitute(table, p, args)).collect();
            let new_ret = substitute(table, ret, args);
            if new_params == params && new_ret == ret {
                ty_id
            } else {
                table.intern(Type::Fn {
                    params: new_params,
                    ret: new_ret,
                })
            }
        }
        Type::ErrorUnion { err, ok } => {
            let new_err = substitute(table, err, args);
            let new_ok = substitute(table, ok, args);
            if new_err == err && new_ok == ok {
                ty_id
            } else {
                table.intern(Type::ErrorUnion {
                    err: new_err,
                    ok: new_ok,
                })
            }
        }
        Type::Named { name, args: old_args } => {
            let new_args: Vec<_> = old_args.iter().map(|&a| substitute(table, a, args)).collect();
            if new_args == old_args {
                ty_id
            } else {
                table.intern(Type::Named {
                    name,
                    args: new_args,
                })
            }
        }
        Type::Int | Type::Float | Type::Bool | Type::Str | Type::Null | Type::Never | Type::Unit => ty_id,
    }
}

pub fn expr_to_type_repr(expr: &Expr) -> Option<TypeRepr> {
    match &expr.kind {
        ExprKind::Name(name) => Some(TypeRepr {
            kind: TypeKind::Named {
                name: name.clone(),
                generics: vec![],
            },
            span: expr.span.clone(),
        }),
        ExprKind::Index { base, index } => {
            let base_name = match &base.kind {
                ExprKind::Name(n) => n.clone(),
                _ => return None,
            };

            let mut generics = Vec::new();
            if let ExprKind::List(items) = &index.kind {
                for item in items {
                    generics.push(expr_to_type_repr(item)?);
                }
            } else {
                generics.push(expr_to_type_repr(index)?);
            }

            Some(TypeRepr {
                kind: TypeKind::Named {
                    name: base_name,
                    generics,
                },
                span: expr.span.clone(),
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::item::Name;
    use crate::diag::Span;

    fn make_name(s: &str) -> Name {
        Name::new(s.to_string(), Span::default())
    }

    fn make_type_repr(name: &str, generics: Vec<TypeRepr>) -> TypeRepr {
        TypeRepr {
            kind: TypeKind::Named {
                name: make_name(name),
                generics,
            },
            span: Span::default(),
        }
    }

    fn make_array_repr(elem: TypeRepr, size: Option<i64>) -> TypeRepr {
        TypeRepr {
            kind: TypeKind::Array {
                elem: Box::new(elem),
                size: size.map(|s| Expr {
                    kind: ExprKind::Int(s),
                    span: Span::default(),
                }),
            },
            span: Span::default(),
        }
    }

    #[test]
    fn test_generic_identity() {
        let mut table = TypeTable::new();
        let arities = HashMap::new();
        table.fresh_var();
        let env = GenericEnv {
            type_params: &["T".to_string()],
            known_arities: &arities,
        };

        // Type `T`
        let repr = make_type_repr("T", vec![]);
        let mut diags = Vec::new();
        let ty_id = check_type_repr(&mut table, &repr, &env, &mut diags);

        assert!(diags.is_empty());
        assert_eq!(table.resolve(ty_id), Type::Var(0));
    }

    #[test]
    fn test_nested_instantiation() {
        let mut table = TypeTable::new();
        let mut arities = HashMap::new();
        table.fresh_var();
        arities.insert("Stack".to_string(), 1);
        let env = GenericEnv {
            type_params: &[],
            known_arities: &arities,
        };

        // Stack[Stack[int]]
        let repr_int = make_type_repr("int", vec![]);
        let repr_inner = make_type_repr("Stack", vec![repr_int]);
        let repr_outer = make_type_repr("Stack", vec![repr_inner]);

        let mut diags = Vec::new();
        let ty_id = check_type_repr(&mut table, &repr_outer, &env, &mut diags);
        assert!(diags.is_empty());

        let ty = table.resolve(ty_id);
        if let Type::Named { name, args } = ty {
            assert_eq!(name, "Stack");
            assert_eq!(args.len(), 1);
            let inner_ty = table.resolve(args[0]);
            if let Type::Named { name: inner_name, args: inner_args } = inner_ty {
                assert_eq!(inner_name, "Stack");
                assert_eq!(inner_args.len(), 1);
                assert_eq!(table.resolve(inner_args[0]), Type::Int);
            } else {
                panic!("Expected inner Type::Named");
            }
        } else {
            panic!("Expected Type::Named");
        }
    }

    #[test]
    fn test_arity_errors() {
        let mut table = TypeTable::new();
        let mut arities = HashMap::new();
        table.fresh_var();
        arities.insert("Map".to_string(), 2);
        let env = GenericEnv {
            type_params: &[],
            known_arities: &arities,
        };

        // Map[int] -> expects 2 args
        let repr = make_type_repr("Map", vec![make_type_repr("int", vec![])]);
        let mut diags = Vec::new();
        check_type_repr(&mut table, &repr, &env, &mut diags);

        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("E0402"));
    }

    #[test]
    fn test_undeclared_parameter() {
        let mut table = TypeTable::new();
        let arities = HashMap::new();
        table.fresh_var();
        let env = GenericEnv {
            type_params: &[], // T is not declared
            known_arities: &arities,
        };

        let repr = make_type_repr("T", vec![]);
        let mut diags = Vec::new();
        check_type_repr(&mut table, &repr, &env, &mut diags);

        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("E0400"));
    }

    #[test]
    fn test_bad_array_size() {
        let mut table = TypeTable::new();
        let arities = HashMap::new();
        table.fresh_var();
        let env = GenericEnv {
            type_params: &["T".to_string()],
            known_arities: &arities,
        };

        let repr_elem = make_type_repr("T", vec![]);
        let repr = make_array_repr(repr_elem, Some(-5)); // size = -5
        let mut diags = Vec::new();
        check_type_repr(&mut table, &repr, &env, &mut diags);

        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("E0404"));
    }

    #[test]
    fn test_use_site_builtin_instantiation() {
        let mut table = TypeTable::new();
        let arities = HashMap::new();
        table.fresh_var();
        let env = GenericEnv {
            type_params: &[],
            known_arities: &arities,
        };

        // int[bool] -> builtin type cannot take arguments
        let repr = make_type_repr("int", vec![make_type_repr("bool", vec![])]);
        let mut diags = Vec::new();
        check_type_repr(&mut table, &repr, &env, &mut diags);

        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("E0403"));
    }

    #[test]
    fn test_expr_to_type_repr() {
        // v.parse[int]() -> [int] is Index { base: ..., index: Name("int") }
        let expr = Expr {
            kind: ExprKind::Index {
                base: Box::new(Expr {
                    kind: ExprKind::Name(make_name("parse")),
                    span: Span::default(),
                }),
                index: Box::new(Expr {
                    kind: ExprKind::Name(make_name("int")),
                    span: Span::default(),
                }),
            },
            span: Span::default(),
        };

        let repr = expr_to_type_repr(&expr).unwrap();
        match repr.kind {
            TypeKind::Named { name, generics } => {
                assert_eq!(name.text, "parse");
                assert_eq!(generics.len(), 1);
                match &generics[0].kind {
                    TypeKind::Named { name: gen_name, generics: sub_gen } => {
                        assert_eq!(gen_name.text, "int");
                        assert!(sub_gen.is_empty());
                    }
                    _ => panic!("Expected Name int"),
                }
            }
            _ => panic!("Expected Named"),
        }
    }
}
