use crate::ast::ty::TypeRepr;

use super::util::Parser;
use super::ParseError;

/// Parse a type: `int`, `T[]`, `int[10]`, `T?`, `(int) int`, `Stack[T]`,
/// `MathError!int`, or bare `!`.
/// TODO(task 08).
pub fn type_repr(_p: &mut Parser<'_>) -> Result<TypeRepr, ParseError> {
    Err(ParseError::new("type parsing not implemented", Default::default()))
}
