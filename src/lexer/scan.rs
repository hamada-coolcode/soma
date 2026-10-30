use super::{LexError, Token};

/// Scan the whole file. TODO(task 01): drive symbol/literal scanners and
/// emit Newline / Indent / Dedent around lines.
pub fn scan_tokens(_src: &str) -> Result<Vec<Token>, LexError> {
    Ok(Vec::new())
}
