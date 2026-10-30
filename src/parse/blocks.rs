use crate::ast::item::Item;
use crate::lexer::Token;

use super::ParseError;

/// Parse a whole `.soma` file into top-level items.
/// TODO(task 06): implement the block structure and delegate to the
/// signature/type/expr parsers.
pub fn parse_file(_toks: &[Token]) -> Result<Vec<Item>, ParseError> {
    Ok(Vec::new())
}
