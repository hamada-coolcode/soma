use crate::diag::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqEq,
    BangEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Assign,
    Arrow,
    FatArrow,
    Dot,
    DotDot,
    Bang,
    Colon,
    Question,
    LParen,
    RParen,
    LBracket,
    RBracket,
    /// `#[` — attribute opener.
    AttrOpen,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FPart {
    Text(String),
    /// Raw source text of the interpolated expression.
    Expr(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokKind {
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
    FStr(Vec<FPart>),
    Op(Op),
    Indent,
    Dedent,
    Newline,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokKind, span: Span) -> Self {
        Self { kind, span }
    }

    // Fixture helpers for tests: all spans are empty by default.
    pub fn ident(s: &str) -> Self {
        Self::new(TokKind::Ident(s.to_string()), Span::default())
    }
    pub fn int(v: i64) -> Self {
        Self::new(TokKind::Int(v), Span::default())
    }
    pub fn float(v: f64) -> Self {
        Self::new(TokKind::Float(v), Span::default())
    }
    pub fn str_(s: &str) -> Self {
        Self::new(TokKind::Str(s.to_string()), Span::default())
    }
    pub fn fstr(parts: Vec<FPart>) -> Self {
        Self::new(TokKind::FStr(parts), Span::default())
    }
    pub fn op(op: Op) -> Self {
        Self::new(TokKind::Op(op), Span::default())
    }
    pub fn indent() -> Self {
        Self::new(TokKind::Indent, Span::default())
    }
    pub fn dedent() -> Self {
        Self::new(TokKind::Dedent, Span::default())
    }
    pub fn newline() -> Self {
        Self::new(TokKind::Newline, Span::default())
    }
    pub fn eof() -> Self {
        Self::new(TokKind::Eof, Span::default())
    }
}
