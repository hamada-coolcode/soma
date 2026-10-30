use crate::ast::item::Name;
use crate::diag::Span;
use crate::lexer::{Op, TokKind, Token};

use super::ParseError;

/// Hand-written recursive-descent cursor over the token stream.
pub struct Parser<'a> {
    pub toks: &'a [Token],
    pub pos: usize,
    /// Lambdas are `x => e`, which collides with `if cond => body`.
    /// Cleared while parsing an `if`/`elif` condition.
    pub allow_lambda: bool,
}

impl<'a> Parser<'a> {
    pub fn new(toks: &'a [Token]) -> Self {
        Self {
            toks,
            pos: 0,
            allow_lambda: true,
        }
    }

    pub fn peek(&self) -> Option<&'a Token> {
        self.toks.get(self.pos)
    }

    pub fn peek_kind(&self) -> Option<&'a TokKind> {
        self.peek().map(|t| &t.kind)
    }

    pub fn peek_span(&self) -> Span {
        self.peek().map(|t| t.span).unwrap_or_default()
    }

    pub fn at_eof(&self) -> bool {
        matches!(self.peek_kind(), None | Some(TokKind::Eof))
    }

    pub fn bump(&mut self) -> Option<&'a Token> {
        let tok = self.toks.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    pub fn check_op(&self, op: Op) -> bool {
        matches!(self.peek_kind(), Some(TokKind::Op(o)) if *o == op)
    }

    pub fn eat_op(&mut self, op: Op) -> bool {
        if self.check_op(op) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    pub fn expect_op(&mut self, op: Op) -> Result<Span, ParseError> {
        if self.check_op(op) {
            let span = self.peek_span();
            self.pos += 1;
            Ok(span)
        } else {
            Err(self.unexpected(&format!("{:?}", op)))
        }
    }

    pub fn at_kw(&self, kw: &str) -> bool {
        matches!(self.peek_kind(), Some(TokKind::Ident(s)) if s == kw)
    }

    pub fn eat_kw(&mut self, kw: &str) -> bool {
        if self.at_kw(kw) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    pub fn expect_ident(&mut self) -> Result<Name, ParseError> {
        match self.peek_kind() {
            Some(TokKind::Ident(text)) => {
                let name = Name::new(text.clone(), self.peek_span());
                self.pos += 1;
                Ok(name)
            }
            _ => Err(self.unexpected("identifier")),
        }
    }

    pub fn skip_newlines(&mut self) {
        while matches!(self.peek_kind(), Some(TokKind::Newline)) {
            self.pos += 1;
        }
    }

    pub fn at_indent(&self) -> bool {
        matches!(self.peek_kind(), Some(TokKind::Indent))
    }

    pub fn expect_indent(&mut self) -> Result<Span, ParseError> {
        if self.at_indent() {
            let span = self.peek_span();
            self.pos += 1;
            Ok(span)
        } else {
            Err(self.unexpected("indented block"))
        }
    }

    pub fn expect_dedent(&mut self) -> Result<Span, ParseError> {
        if matches!(self.peek_kind(), Some(TokKind::Dedent)) {
            let span = self.peek_span();
            self.pos += 1;
            Ok(span)
        } else {
            Err(self.unexpected("end of block"))
        }
    }

    pub fn unexpected(&self, expected: &str) -> ParseError {
        let found = match self.peek_kind() {
            None | Some(TokKind::Eof) => "end of input".to_string(),
            Some(kind) => format!("{:?}", kind),
        };
        ParseError {
            message: format!("expected {}, found {}", expected, found),
            span: self.peek_span(),
            expected: vec![expected.to_string()],
        }
    }
}
