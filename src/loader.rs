//! Module loading for `import` trees (owned by task 17).

use crate::diag::{Diagnostic, ErrorKind, Span};

#[derive(Debug, Default)]
pub struct Loaded {
    pub files: Vec<LoadedFile>,
}

#[derive(Debug)]
pub struct LoadedFile {
    pub path: String,
    pub source: String,
}

pub fn load_entry(_path: &str) -> Result<Loaded, Diagnostic> {
    Err(Diagnostic::new(
        ErrorKind::Io,
        "module loading not implemented",
        Span::default(),
    ))
}
