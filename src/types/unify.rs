use super::ty::{TypeId, TypeTable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnifyError {
    pub message: String,
}

/// Unify two types, binding type variables as needed (owned by task 10).
pub fn unify(_table: &mut TypeTable, _a: TypeId, _b: TypeId) -> Result<(), UnifyError> {
    Ok(())
}
