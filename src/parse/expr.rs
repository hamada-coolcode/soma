use crate::ast::expr::{Expr, Stmt};

use super::ParseError;
use super::util::Parser;

/// Parse an expression with precedence climbing (see jules/grammar.ebnf).
/// TODO(task 09/10).
pub fn expr(_p: &mut Parser<'_>, _min_prec: u8) -> Result<Expr, ParseError> {
    Err(ParseError::new(
        "expression parsing not implemented",
        Default::default(),
    ))
}

/// Parse one statement (assignment, control flow, return, or expression).
/// TODO(task 09/10).
pub fn stmt(_p: &mut Parser<'_>) -> Result<Stmt, ParseError> {
    Err(ParseError::new(
        "statement parsing not implemented",
        Default::default(),
    ))
}
