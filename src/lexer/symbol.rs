use super::{Op, Token};

/// Reserved words. These are still emitted as `TokKind::Ident`; the parser
/// distinguishes keywords by string match.
pub const KEYWORDS: &[&str] = &[
    "fn", "object", "trait", "error", "enum", "import", "const", "test", "export", "if", "elif",
    "else", "while", "for", "in", "loop", "break", "continue", "return", "match", "and", "or",
    "not", "true", "false", "null", "self",
];

pub fn is_keyword(s: &str) -> bool {
    KEYWORDS.contains(&s)
}

/// Scan a symbolic operator starting at `start` using maximal munch.
/// Returns the operator and the byte offset just past it.
pub fn scan_op(_src: &str, _start: usize) -> Option<(Op, usize)> {
    None
}

/// Build an operator token over `[start, end)`.
pub fn op_token(op: Op, start: usize, end: usize) -> Token {
    Token::new(
        super::TokKind::Op(op),
        crate::diag::Span::new(start, end),
    )
}
