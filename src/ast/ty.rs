use crate::diag::Span;

use super::expr::Expr;
use super::item::Name;

#[derive(Debug, Clone, PartialEq)]
pub struct TypeRepr {
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    /// `int`, `Shape`, `Stack[T]`
    Named { name: Name, generics: Vec<TypeRepr> },
    /// `T[]` (size `None`) or `int[10]` (size `Some`)
    Array {
        elem: Box<TypeRepr>,
        size: Option<Expr>,
    },
    /// `T?`
    Optional(Box<TypeRepr>),
    /// `(int) int`
    Fn {
        params: Vec<TypeRepr>,
        ret: Box<TypeRepr>,
    },
    /// `MathError!int`
    ErrorUnion {
        err: Box<TypeRepr>,
        ok: Box<TypeRepr>,
    },
    /// bare `!` — fallible, no value
    Never,
}
