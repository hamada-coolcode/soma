use crate::diag::Span;

use super::{LexError, Token};

/// Tracks the indentation stack for a file and emits Indent/Dedent tokens.
///
/// Contract (see jules/grammar.ebnf):
/// - widths are compared by value, not by column count arithmetic
/// - a deeper line emits one Indent
/// - a shallower line emits one Dedent per level popped
/// - a width that matches no open level is an error
/// - tabs are rejected
#[derive(Debug, Default)]
pub struct IndentTracker {
    /// Open indentation widths; starts with 0 for the file body.
    pub levels: Vec<usize>,
}

impl IndentTracker {
    pub fn new() -> Self {
        Self { levels: vec![0] }
    }

    /// Feed the measured indentation of a new line. Returns the
    /// Indent/Dedent tokens that must be emitted before the line's tokens.
    pub fn feed(&mut self, width: usize, span: Span) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        let top = *self.levels.last().unwrap();

        if width > top {
            self.levels.push(width);
            tokens.push(Token::new(crate::lexer::TokKind::Indent, span));
        } else if width < top {
            while let Some(&last) = self.levels.last() {
                if last > width {
                    self.levels.pop();
                    tokens.push(Token::new(crate::lexer::TokKind::Dedent, span));
                } else {
                    break;
                }
            }
            if *self.levels.last().unwrap() != width {
                return Err(LexError {
                    message: format!("indentation width {} matches no open level", width),
                    span,
                });
            }
        }

        Ok(tokens)
    }

    /// Emit the Dedents needed at end of file.
    pub fn finish(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while self.levels.len() > 1 {
            self.levels.pop();
            tokens.push(Token::new(crate::lexer::TokKind::Dedent, Span::default()));
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_tracker() {
        let mut tracker = IndentTracker::new();
        let span = Span::new(0, 0);

        // equal
        let tokens = tracker.feed(0, span).unwrap();
        assert!(tokens.is_empty());

        // deeper
        let tokens = tracker.feed(2, span).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, crate::lexer::TokKind::Indent);
        assert_eq!(tracker.levels, vec![0, 2]);

        // deeper again
        let tokens = tracker.feed(4, span).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, crate::lexer::TokKind::Indent);
        assert_eq!(tracker.levels, vec![0, 2, 4]);

        // shallower
        let tokens = tracker.feed(0, span).unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, crate::lexer::TokKind::Dedent);
        assert_eq!(tokens[1].kind, crate::lexer::TokKind::Dedent);
        assert_eq!(tracker.levels, vec![0]);

        // bad width
        tracker.feed(2, span).unwrap();
        tracker.feed(4, span).unwrap();
        let res = tracker.feed(3, span);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().message,
            "indentation width 3 matches no open level"
        );
    }

    #[test]
    fn test_indent_tracker_finish() {
        let mut tracker = IndentTracker::new();
        let span = Span::new(0, 0);
        tracker.feed(2, span).unwrap();
        tracker.feed(4, span).unwrap();
        let tokens = tracker.finish();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, crate::lexer::TokKind::Dedent);
        assert_eq!(tokens[1].kind, crate::lexer::TokKind::Dedent);
        assert_eq!(tracker.levels, vec![0]);
    }
}
