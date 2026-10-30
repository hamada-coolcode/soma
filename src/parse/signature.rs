use crate::ast::item::{Name, Param};
use crate::ast::ty::TypeRepr;

use super::ParseError;
use super::util::Parser;

/// A parsed declaration signature: `name [generics] params -> ret`.
#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    pub name: Name,
    pub generics: Vec<Name>,
    pub params: Vec<Param>,
    pub ret: Option<TypeRepr>,
}

/// Parse `name [G] p: T p: T -> R` (the caller has already consumed nothing).
/// TODO(task 07).
pub fn signature(_p: &mut Parser<'_>) -> Result<Signature, ParseError> {
    Err(ParseError::new(
        "signature parsing not implemented",
        Default::default(),
    ))
}
