//! Builtin function registry (owned by task 08).

use crate::ast::ty::TypeRepr;

#[derive(Debug, Clone, Copy)]
pub struct Builtin {
    /// Fully qualified path, e.g. `io.term.output`.
    pub path: &'static str,
    /// Signature in soma syntax, e.g. `s: string -> !`.
    pub sig: &'static str,
}

pub const BUILTINS: &[Builtin] = &[];

pub fn find(path: &str) -> Option<&'static Builtin> {
    BUILTINS.iter().find(|b| b.path == path)
}

/// Parse a type written in soma syntax, using the real type parser.
pub fn type_from_str(src: &str) -> Option<TypeRepr> {
    let toks = crate::lexer::scan(src).ok()?;
    let mut p = crate::parse::util::Parser::new(&toks);
    crate::parse::types::type_repr(&mut p).ok()
}
