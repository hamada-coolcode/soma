use super::{LexError, Token};

/// Scan a numeric literal starting at `start` (ASCII digit).
/// Returns the token and the byte offset just past it.
pub fn scan_number(_src: &str, _start: usize) -> Option<(Token, usize)> {
    None
}

/// Scan a plain string literal (`"..."` or `"""..."""`) starting at `start`.
pub fn scan_string(_src: &str, _start: usize) -> Result<Option<(Token, usize)>, LexError> {
    Ok(None)
}

/// Scan an interpolated string literal (`f"..."`) starting at `start`.
/// The result is `TokKind::FStr` with text/expression parts split.
pub fn scan_fstring(_src: &str, _start: usize) -> Result<Option<(Token, usize)>, LexError> {
    Ok(None)
}
