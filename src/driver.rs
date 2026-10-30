//! Pipeline entry points used by the CLI and tests (owned by task 01).

use crate::ast::item::Item;
use crate::diag::{Diagnostic, ErrorKind, SourceMap};

pub struct Parsed {
    pub name: String,
    pub source: String,
    pub items: Vec<Item>,
}

/// Lex + parse one source file. Returns a single diagnostic on failure.
pub fn parse_source(name: &str, src: &str) -> Result<Parsed, Diagnostic> {
    let toks =
        crate::lexer::scan(src).map_err(|e| Diagnostic::new(ErrorKind::Lex, e.message, e.span))?;
    let items = crate::parse::parse_file(&toks).map_err(|e| {
        let d = Diagnostic::new(ErrorKind::Parse, e.message, e.span);
        if e.expected.is_empty() {
            d
        } else {
            d.with_help(format!("expected: {}", e.expected.join(", ")))
        }
    })?;
    Ok(Parsed {
        name: name.to_string(),
        source: src.to_string(),
        items,
    })
}

/// Parse + resolve + type check. Returns every diagnostic found.
pub fn check_source(name: &str, src: &str) -> (Option<Parsed>, Vec<Diagnostic>) {
    match parse_source(name, src) {
        Err(d) => (None, vec![d]),
        Ok(parsed) => {
            let mut diags = crate::resolve::check(&parsed.items);
            diags.extend(crate::types::check(&parsed.items));
            (Some(parsed), diags)
        }
    }
}

/// Lower the AST and print `.psoma`.
pub fn emit_psoma(parsed: &Parsed) -> String {
    let lowered = crate::desugar::lower(parsed.items.clone());
    crate::desugar::psoma::emit(&lowered)
}

/// Render a diagnostic against the file it came from.
pub fn render(diag: &Diagnostic, name: &str, src: &str) -> String {
    let mut map = SourceMap::new();
    let _id = map.add(name.to_string(), src.to_string());
    crate::diag::render::render(diag, &map)
}
