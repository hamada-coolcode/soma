use super::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagCode {
    E0100, // lex
    E0200, // parse
    E0300, // resolve
    E0301, // read-only parameter
    E0302, // undefined name
    E0303, // duplicate definition
    E0400, // type
    E0401, // mismatch
    E0402, // not callable
    E0403, // wrong arity
    E0500, // match
    E0501, // non-exhaustive
}

impl DiagCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::E0100 => "E0100",
            Self::E0200 => "E0200",
            Self::E0300 => "E0300",
            Self::E0301 => "E0301",
            Self::E0302 => "E0302",
            Self::E0303 => "E0303",
            Self::E0400 => "E0400",
            Self::E0401 => "E0401",
            Self::E0402 => "E0402",
            Self::E0403 => "E0403",
            Self::E0500 => "E0500",
            Self::E0501 => "E0501",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "E0100" => Some(Self::E0100),
            "E0200" => Some(Self::E0200),
            "E0300" => Some(Self::E0300),
            "E0301" => Some(Self::E0301),
            "E0302" => Some(Self::E0302),
            "E0303" => Some(Self::E0303),
            "E0400" => Some(Self::E0400),
            "E0401" => Some(Self::E0401),
            "E0402" => Some(Self::E0402),
            "E0403" => Some(Self::E0403),
            "E0500" => Some(Self::E0500),
            "E0501" => Some(Self::E0501),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Lex,
    Parse,
    Type,
    Memory,
    Io,
    Other,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::Lex => write!(f, "Lex Error"),
            ErrorKind::Parse => write!(f, "Parse Error"),
            ErrorKind::Type => write!(f, "Type Error"),
            ErrorKind::Memory => write!(f, "Memory Error"),
            ErrorKind::Io => write!(f, "IO Error"),
            ErrorKind::Other => write!(f, "Error"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Span,
    pub help: Option<String>,
    pub code: Option<DiagCode>,
}

impl Diagnostic {
    pub fn new(kind: ErrorKind, message: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
            help: None,
            code: None,
        }
    }

    pub fn lex(message: impl Into<String>, span: Span) -> Self {
        Self::new(ErrorKind::Lex, message, span).with_code(DiagCode::E0100)
    }

    pub fn parse(message: impl Into<String>, span: Span) -> Self {
        Self::new(ErrorKind::Parse, message, span).with_code(DiagCode::E0200)
    }

    pub fn with_code(mut self, code: DiagCode) -> Self {
        self.code = Some(code);
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}
