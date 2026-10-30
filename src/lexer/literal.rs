use super::{LexError, Token};
use crate::diag::Span;
use crate::lexer::token::{TokKind, FPart};

/// Scan a numeric literal starting at `start` (ASCII digit).
/// Returns the token and the byte offset just past it.
pub fn scan_number(src: &str, start: usize) -> Option<(Token, usize)> {
    let bytes = src.as_bytes();
    if start >= bytes.len() || !bytes[start].is_ascii_digit() {
        return None;
    }

    let mut end = start;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }

    let mut is_float = false;
    if end < bytes.len() && bytes[end] == b'.' {
        let after_dot = end + 1;
        if after_dot < bytes.len() && bytes[after_dot].is_ascii_digit() {
            is_float = true;
            end = after_dot;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
        }
    }

    let span = Span::new(start, end);
    let substr = &src[start..end];
    let token = if is_float {
        let val: f64 = substr.parse().unwrap_or(0.0);
        Token::new(TokKind::Float(val), span)
    } else {
        let val: i64 = substr.parse().unwrap_or(i64::MAX);
        Token::new(TokKind::Int(val), span)
    };

    Some((token, end))
}

/// Scan a plain string literal (`"..."` or `"""..."""`) starting at `start`.
pub fn scan_string(src: &str, start: usize) -> Result<Option<(Token, usize)>, LexError> {
    let bytes = src.as_bytes();
    if start >= bytes.len() || bytes[start] != b'"' {
        return Ok(None);
    }

    if start + 2 < bytes.len() && bytes[start..start + 3] == *b"\"\"\"" {
        // Multiline string
        let mut end = start + 3;
        while end + 2 < bytes.len() {
            if bytes[end..end + 3] == *b"\"\"\"" {
                let text = &src[start + 3..end];
                let end = end + 3;
                let span = Span::new(start, end);
                return Ok(Some((Token::new(TokKind::Str(text.to_string()), span), end)));
            }
            end += 1;
        }
        return Err(LexError {
            message: "Unterminated multiline string".to_string(),
            span: Span::new(start, bytes.len()),
        });
    } else {
        // Standard string
        let mut end = start + 1;
        let mut text = String::new();
        while end < bytes.len() {
            match bytes[end] {
                b'"' => {
                    let end = end + 1;
                    let span = Span::new(start, end);
                    return Ok(Some((Token::new(TokKind::Str(text), span), end)));
                }
                b'\n' | b'\r' => {
                    return Err(LexError {
                        message: "Unterminated string literal".to_string(),
                        span: Span::new(start, end),
                    });
                }
                b'\\' => {
                    end += 1;
                    if end >= bytes.len() {
                        return Err(LexError {
                            message: "Unterminated string literal".to_string(),
                            span: Span::new(start, bytes.len()),
                        });
                    }
                    match bytes[end] {
                        b'n' => text.push('\n'),
                        b't' => text.push('\t'),
                        b'r' => text.push('\r'),
                        b'\\' => text.push('\\'),
                        b'"' => text.push('"'),
                        b'{' => text.push('{'),
                        b'}' => text.push('}'),
                        _ => {
                            let ch = src[end..].chars().next().unwrap();
                            text.push(ch);
                            end += ch.len_utf8() - 1;
                        }
                    }
                }
                _ => {
                    let ch = src[end..].chars().next().unwrap();
                    text.push(ch);
                    end += ch.len_utf8() - 1;
                }
            }
            end += 1;
        }
        return Err(LexError {
            message: "Unterminated string literal".to_string(),
            span: Span::new(start, bytes.len()),
        });
    }
}

/// Scan an interpolated string literal (`f"..."`) starting at `start`.
/// The result is `TokKind::FStr` with text/expression parts split.
pub fn scan_fstring(src: &str, start: usize) -> Result<Option<(Token, usize)>, LexError> {
    let bytes = src.as_bytes();
    if start + 1 >= bytes.len() || bytes[start] != b'f' || bytes[start + 1] != b'"' {
        return Ok(None);
    }

    let mut end = start + 2;
    let mut parts = Vec::new();
    let mut current_text = String::new();

    while end < bytes.len() {
        match bytes[end] {
            b'"' => {
                if !current_text.is_empty() {
                    parts.push(FPart::Text(current_text));
                }
                end += 1;
                let span = Span::new(start, end);
                return Ok(Some((Token::new(TokKind::FStr(parts), span), end)));
            }
            b'\n' | b'\r' => {
                return Err(LexError {
                    message: "Unterminated f-string".to_string(),
                    span: Span::new(start, end),
                });
            }
            b'\\' => {
                end += 1;
                if end >= bytes.len() {
                    return Err(LexError {
                        message: "Unterminated f-string".to_string(),
                        span: Span::new(start, bytes.len()),
                    });
                }
                match bytes[end] {
                    b'n' => current_text.push('\n'),
                    b't' => current_text.push('\t'),
                    b'r' => current_text.push('\r'),
                    b'\\' => current_text.push('\\'),
                    b'"' => current_text.push('"'),
                    b'{' => current_text.push('{'),
                    b'}' => current_text.push('}'),
                    _ => {
                        let ch = src[end..].chars().next().unwrap();
                        current_text.push(ch);
                        end += ch.len_utf8() - 1;
                    }
                }
            }
            b'{' => {
                if !current_text.is_empty() {
                    parts.push(FPart::Text(current_text.clone()));
                    current_text.clear();
                }
                end += 1;
                let expr_start = end;
                let mut depth = 1;
                let mut in_string = false;
                let mut string_delim = 0u8;

                while end < bytes.len() && depth > 0 {
                    if in_string {
                        if bytes[end] == b'\\' {
                            end += 1;
                        } else if bytes[end] == string_delim {
                            in_string = false;
                        }
                    } else {
                        match bytes[end] {
                            b'"' => {
                                in_string = true;
                                string_delim = b'"';
                            }
                            b'{' => depth += 1,
                            b'}' => depth -= 1,
                            _ => {}
                        }
                    }
                    if depth > 0 {
                        end += 1;
                    }
                }

                if depth > 0 {
                    return Err(LexError {
                        message: "Unterminated expression in f-string".to_string(),
                        span: Span::new(expr_start - 1, bytes.len()),
                    });
                }

                parts.push(FPart::Expr(src[expr_start..end].to_string()));
            }
            b'}' => {
                current_text.push('}');
            }
            _ => {
                let ch = src[end..].chars().next().unwrap();
                current_text.push(ch);
                end += ch.len_utf8() - 1;
            }
        }
        end += 1;
    }

    Err(LexError {
        message: "Unterminated f-string".to_string(),
        span: Span::new(start, bytes.len()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::{TokKind, FPart};
    use crate::diag::Span;

    #[test]
    fn test_scan_number() {
        assert_eq!(scan_number("123", 0), Some((Token::new(TokKind::Int(123), Span::new(0, 3)), 3)));
        assert_eq!(scan_number(" 123", 1), Some((Token::new(TokKind::Int(123), Span::new(1, 4)), 4)));
        assert_eq!(scan_number("123.456", 0), Some((Token::new(TokKind::Float(123.456), Span::new(0, 7)), 7)));
        assert_eq!(scan_number("abc", 0), None);
        // large number fallback
        assert_eq!(scan_number("999999999999999999999999999999999999", 0), Some((Token::new(TokKind::Int(i64::MAX), Span::new(0, 36)), 36)));
    }

    #[test]
    fn test_scan_string() {
        // Normal string
        assert_eq!(
            scan_string(r#""hello""#, 0).unwrap(),
            Some((Token::new(TokKind::Str("hello".to_string()), Span::new(0, 7)), 7))
        );
        // Escapes
        assert_eq!(
            scan_string(r#""\n\t\"""#, 0).unwrap(),
            Some((Token::new(TokKind::Str("\n\t\"".to_string()), Span::new(0, 8)), 8))
        );
        // Multiline string
        assert_eq!(
            scan_string(r#""""hello\nworld""""#, 0).unwrap(),
            Some((Token::new(TokKind::Str(r#"hello\nworld"#.to_string()), Span::new(0, 18)), 18))
        );
        // Unterminated string
        assert!(scan_string(r#""unterminated"#, 0).is_err());
        assert!(scan_string(r#""""unterminated"#, 0).is_err());

        // Multibyte escapes should not crash
        assert_eq!(
            scan_string("\"\\💖\"", 0).unwrap(),
            Some((Token::new(TokKind::Str("💖".to_string()), Span::new(0, 7)), 7))
        );
    }

    #[test]
    fn test_scan_fstring() {
        assert_eq!(
            scan_fstring(r#"f"hello""#, 0).unwrap(),
            Some((Token::new(TokKind::FStr(vec![FPart::Text("hello".to_string())]), Span::new(0, 8)), 8))
        );
        assert_eq!(
            scan_fstring(r#"f"hello {world}""#, 0).unwrap(),
            Some((Token::new(TokKind::FStr(vec![
                FPart::Text("hello ".to_string()),
                FPart::Expr("world".to_string())
            ]), Span::new(0, 16)), 16))
        );
        assert_eq!(
            scan_fstring(r#"f"{a + { b }}""#, 0).unwrap(),
            Some((Token::new(TokKind::FStr(vec![
                FPart::Expr("a + { b }".to_string())
            ]), Span::new(0, 14)), 14))
        );
        // Unterminated expr
        assert!(scan_fstring(r#"f"{abc"#, 0).is_err());
        // String inside expr
        assert_eq!(
            scan_fstring(r#"f"hi {a + "}"} there""#, 0).unwrap(),
            Some((Token::new(TokKind::FStr(vec![
                FPart::Text("hi ".to_string()),
                FPart::Expr(r#"a + "}""#.to_string()),
                FPart::Text(" there".to_string())
            ]), Span::new(0, 21)), 21))
        );
        // Multibyte escapes should not crash
        assert_eq!(
            scan_fstring("f\"\\💖\"", 0).unwrap(),
            Some((Token::new(TokKind::FStr(vec![FPart::Text("💖".to_string())]), Span::new(0, 8)), 8))
        );
    }
}
