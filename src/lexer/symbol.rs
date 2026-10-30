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
pub fn scan_op(src: &str, start: usize) -> Option<(Op, usize)> {
    let tail = &src[start..];

    // 2-char operators
    if tail.starts_with("#[") {
        return Some((Op::AttrOpen, start + 2));
    }
    if tail.starts_with("->") {
        return Some((Op::Arrow, start + 2));
    }
    if tail.starts_with("=>") {
        return Some((Op::FatArrow, start + 2));
    }
    if tail.starts_with("==") {
        return Some((Op::EqEq, start + 2));
    }
    if tail.starts_with("!=") {
        return Some((Op::BangEq, start + 2));
    }
    if tail.starts_with("<=") {
        return Some((Op::LtEq, start + 2));
    }
    if tail.starts_with(">=") {
        return Some((Op::GtEq, start + 2));
    }
    if tail.starts_with("..") {
        return Some((Op::DotDot, start + 2));
    }

    // 1-char operators
    let mut chars = tail.chars();
    match chars.next() {
        Some('+') => Some((Op::Plus, start + 1)),
        Some('-') => Some((Op::Minus, start + 1)),
        Some('*') => Some((Op::Star, start + 1)),
        Some('/') => Some((Op::Slash, start + 1)),
        Some('%') => Some((Op::Percent, start + 1)),
        Some('=') => Some((Op::Assign, start + 1)),
        Some('<') => Some((Op::Lt, start + 1)),
        Some('>') => Some((Op::Gt, start + 1)),
        Some('!') => Some((Op::Bang, start + 1)),
        Some(':') => Some((Op::Colon, start + 1)),
        Some('?') => Some((Op::Question, start + 1)),
        Some('(') => Some((Op::LParen, start + 1)),
        Some(')') => Some((Op::RParen, start + 1)),
        Some('[') => Some((Op::LBracket, start + 1)),
        Some(']') => Some((Op::RBracket, start + 1)),
        Some('.') => Some((Op::Dot, start + 1)),
        _ => None,
    }
}

/// Build an operator token over `[start, end)`.
pub fn op_token(op: Op, start: usize, end: usize) -> Token {
    Token::new(super::TokKind::Op(op), crate::diag::Span::new(start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_op() {
        let cases = [
            ("#[", Some((Op::AttrOpen, 2))),
            ("->", Some((Op::Arrow, 2))),
            ("=>", Some((Op::FatArrow, 2))),
            ("==", Some((Op::EqEq, 2))),
            ("!=", Some((Op::BangEq, 2))),
            ("<=", Some((Op::LtEq, 2))),
            (">=", Some((Op::GtEq, 2))),
            ("..", Some((Op::DotDot, 2))),
            ("+", Some((Op::Plus, 1))),
            ("-", Some((Op::Minus, 1))),
            ("*", Some((Op::Star, 1))),
            ("/", Some((Op::Slash, 1))),
            ("%", Some((Op::Percent, 1))),
            ("=", Some((Op::Assign, 1))),
            ("<", Some((Op::Lt, 1))),
            (">", Some((Op::Gt, 1))),
            ("!", Some((Op::Bang, 1))),
            (":", Some((Op::Colon, 1))),
            ("?", Some((Op::Question, 1))),
            ("(", Some((Op::LParen, 1))),
            (")", Some((Op::RParen, 1))),
            ("[", Some((Op::LBracket, 1))),
            ("]", Some((Op::RBracket, 1))),
            (".", Some((Op::Dot, 1))),
            ("#", None),
            ("x", None),
            ("->x", Some((Op::Arrow, 2))),
            ("-x", Some((Op::Minus, 1))),
            ("..x", Some((Op::DotDot, 2))),
            (".x", Some((Op::Dot, 1))),
            ("==x", Some((Op::EqEq, 2))),
            ("=x", Some((Op::Assign, 1))),
            ("#[x", Some((Op::AttrOpen, 2))),
        ];

        for (input, expected) in cases {
            assert_eq!(scan_op(input, 0), expected, "failed on {}", input);
        }

        assert_eq!(scan_op("a+b", 1), Some((Op::Plus, 2)));
        assert_eq!(scan_op("a->b", 1), Some((Op::Arrow, 3)));
    }

    #[test]
    fn test_is_keyword() {
        assert!(is_keyword("fn"));
        assert!(is_keyword("true"));
        assert!(!is_keyword("fnx"));
        assert!(!is_keyword("function"));
    }
}
