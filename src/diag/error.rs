use super::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Lex,
    Parse,
    Type,
    Memory,
    Io,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Span,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn new(kind: ErrorKind, message: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
            help: None,
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}
