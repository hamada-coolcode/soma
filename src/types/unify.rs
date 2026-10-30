use super::ty::{Type, TypeId, TypeTable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnifyError {
    pub message: String,
}

/// Unify two types, binding type variables as needed (owned by task 10).
pub fn unify(table: &mut TypeTable, a_id: TypeId, b_id: TypeId) -> Result<(), UnifyError> {
    let a = table.resolve(a_id);
    let b = table.resolve(b_id);

    if a == b {
        return Ok(());
    }

    match (a.clone(), b.clone()) {
        (Type::Var(v), _) => unify_var(table, v, b_id),
        (_, Type::Var(v)) => unify_var(table, v, a_id),
        (Type::List(elem_a), Type::List(elem_b)) => unify(table, elem_a, elem_b),
        (Type::Array(elem_a, size_a), Type::Array(elem_b, size_b)) if size_a == size_b => {
            unify(table, elem_a, elem_b)
        }
        (Type::Optional(elem_a), Type::Optional(elem_b)) => unify(table, elem_a, elem_b),
        (Type::ErrorUnion { err: err_a, ok: ok_a }, Type::ErrorUnion { err: err_b, ok: ok_b }) => {
            unify(table, err_a, err_b)?;
            unify(table, ok_a, ok_b)
        }
        (Type::Fn { params: params_a, ret: ret_a }, Type::Fn { params: params_b, ret: ret_b }) => {
            if params_a.len() != params_b.len() {
                return Err(UnifyError {
                    message: format!("Cannot unify {} and {}", table.display(&a), table.display(&b)),
                });
            }
            for (pa, pb) in params_a.iter().zip(params_b.iter()) {
                unify(table, *pa, *pb)?;
            }
            unify(table, ret_a, ret_b)
        }
        (Type::Named { name: name_a, args: args_a }, Type::Named { name: name_b, args: args_b }) => {
            if name_a != name_b || args_a.len() != args_b.len() {
                return Err(UnifyError {
                    message: format!("Cannot unify {} and {}", table.display(&a), table.display(&b)),
                });
            }
            for (aa, ab) in args_a.iter().zip(args_b.iter()) {
                unify(table, *aa, *ab)?;
            }
            Ok(())
        }
        _ => Err(UnifyError {
            message: format!("Cannot unify {} and {}", table.display(&a), table.display(&b)),
        }),
    }
}

fn unify_var(table: &mut TypeTable, var: u32, ty_id: TypeId) -> Result<(), UnifyError> {
    let ty = table.resolve(ty_id);

    // Occurs check
    if occurs(table, var, ty_id) {
        return Err(UnifyError {
            message: format!("Recursive type binding: _ occurs in {}", table.display(&ty)),
        });
    }

    table.set_var(var, ty);
    Ok(())
}

fn occurs(table: &TypeTable, var: u32, ty_id: TypeId) -> bool {
    let ty = table.resolve(ty_id);
    match ty {
        Type::Var(v) => v == var,
        Type::List(elem) | Type::Optional(elem) | Type::Array(elem, _) => occurs(table, var, elem),
        Type::ErrorUnion { err, ok } => occurs(table, var, err) || occurs(table, var, ok),
        Type::Fn { params, ret } => {
            params.iter().any(|&p| occurs(table, var, p)) || occurs(table, var, ret)
        }
        Type::Named { args, .. } => args.iter().any(|&a| occurs(table, var, a)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unify_basic() {
        let mut table = TypeTable::new();
        let t_int = table.intern(Type::Int);
        let t_float = table.intern(Type::Float);
        assert!(unify(&mut table, t_int, t_int).is_ok());
        assert!(unify(&mut table, t_int, t_float).is_err());
    }

    #[test]
    fn test_unify_var_binding() {
        let mut table = TypeTable::new();
        let v1 = table.fresh_var();
        let t_int = table.intern(Type::Int);

        assert!(unify(&mut table, v1, t_int).is_ok());
        assert_eq!(table.resolve(v1), Type::Int);
    }

    #[test]
    fn test_unify_occurs_check() {
        let mut table = TypeTable::new();
        let v1 = table.fresh_var();
        let t_list = table.intern(Type::List(v1));

        let res = unify(&mut table, v1, t_list);
        assert!(res.is_err());
        assert!(res.unwrap_err().message.contains("Recursive type binding"));
    }

    #[test]
    fn test_unify_structural_list() {
        let mut table = TypeTable::new();
        let t_int = table.intern(Type::Int);
        let t_float = table.intern(Type::Float);
        let l1 = table.intern(Type::List(t_int));
        let l2 = table.intern(Type::List(t_int));
        let l3 = table.intern(Type::List(t_float));

        assert!(unify(&mut table, l1, l2).is_ok());
        assert!(unify(&mut table, l1, l3).is_err());
    }

    #[test]
    fn test_unify_structural_fn() {
        let mut table = TypeTable::new();
        let t_int = table.intern(Type::Int);
        let t_float = table.intern(Type::Float);
        let f1 = table.intern(Type::Fn { params: vec![t_int], ret: t_float });
        let f2 = table.intern(Type::Fn { params: vec![t_int], ret: t_float });
        let f3 = table.intern(Type::Fn { params: vec![t_float], ret: t_float });
        let f4 = table.intern(Type::Fn { params: vec![t_int, t_int], ret: t_float });

        assert!(unify(&mut table, f1, f2).is_ok());
        assert!(unify(&mut table, f1, f3).is_err());
        assert!(unify(&mut table, f1, f4).is_err());
    }

    #[test]
    fn test_unify_named_mismatch() {
        let mut table = TypeTable::new();
        let t_int = table.intern(Type::Int);
        let n1 = table.intern(Type::Named { name: "A".to_string(), args: vec![t_int] });
        let n2 = table.intern(Type::Named { name: "B".to_string(), args: vec![t_int] });
        assert!(unify(&mut table, n1, n2).is_err());
    }

    #[test]
    fn test_unify_transitive_vars() {
        let mut table = TypeTable::new();
        let v1 = table.fresh_var();
        let v2 = table.fresh_var();
        let t_str = table.intern(Type::Str);

        assert!(unify(&mut table, v1, v2).is_ok());
        assert!(unify(&mut table, v2, t_str).is_ok());

        assert_eq!(table.resolve(v1), Type::Str);
        assert_eq!(table.resolve(v2), Type::Str);
    }
}
