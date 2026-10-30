use super::{Diagnostic, SourceMap};
use ariadne::{Cache, Label, Report, ReportKind, Source};
use std::collections::HashMap;
use std::panic;

struct SourceMapCache<'a> {
    map: &'a SourceMap,
    sources: HashMap<usize, Source>,
}

impl<'a> Cache<usize> for SourceMapCache<'a> {
    type Storage = String;

    fn fetch(&mut self, id: &usize) -> Result<&Source, Box<dyn std::fmt::Debug + '_>> {
        if !self.sources.contains_key(id) {
            // We use catch_unwind to see if the file exists
            let text =
                panic::catch_unwind(panic::AssertUnwindSafe(|| self.map.source(*id).to_string()))
                    .map_err(|_| Box::new("File not found") as Box<dyn std::fmt::Debug>)?;
            self.sources.insert(*id, Source::from(text));
        }
        Ok(self.sources.get(id).unwrap())
    }

    fn display<'b>(&self, id: &'b usize) -> Option<Box<dyn std::fmt::Display + 'b>> {
        let name =
            panic::catch_unwind(panic::AssertUnwindSafe(|| self.map.name(*id).to_string())).ok()?;
        Some(Box::new(name))
    }
}

pub fn render(diag: &Diagnostic, map: &SourceMap) -> String {
    // Find the file that contains this span.
    // If we have multiple files, we assume the one whose text is large enough to contain the span.
    // Ideally we would have a file_id in the Diagnostic, but we don't.
    // Wait! Is there a global file offset logic?
    // No, `sourcemap.add` just pushes.
    let mut file_id = 0;
    for id in 0.. {
        let text = panic::catch_unwind(panic::AssertUnwindSafe(|| map.source(id)));
        if let Ok(t) = text {
            if diag.span.end <= t.len() {
                file_id = id;
                break;
            }
        } else {
            break;
        }
    }

    let mut cache = SourceMapCache {
        map,
        sources: HashMap::new(),
    };

    let span = (file_id, diag.span.start..diag.span.end);
    let mut report = Report::build(ReportKind::Error, span)
        .with_message(&diag.message)
        .with_label(
            Label::new((file_id, diag.span.start..diag.span.end)).with_message(&diag.message),
        );

    if let Some(help) = &diag.help {
        report = report.with_help(help.clone());
    }

    let mut vec = Vec::new();
    report.finish().write(&mut cache, &mut vec).unwrap();
    String::from_utf8(vec).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diag::{ErrorKind, Span};

    #[test]
    fn test_render_lex_error() {
        let mut map = SourceMap::new();
        map.add("test.soma", "let x = 1;\nlet y = 2;\n");
        let diag = Diagnostic::new(ErrorKind::Lex, "Invalid character", Span::new(4, 5))
            .with_help("Remove the invalid character");
        let output = render(&diag, &map);

        assert!(output.contains("Invalid character"));
        assert!(output.contains("test.soma"));
        assert!(output.contains("1:5")); // line 1 col 5
        assert!(output.contains("Remove the invalid character"));
    }

    #[test]
    fn test_render_all_error_kinds() {
        let mut map = SourceMap::new();
        map.add("test2.soma", "fn main() {}");
        let kinds = vec![
            ErrorKind::Lex,
            ErrorKind::Parse,
            ErrorKind::Type,
            ErrorKind::Memory,
            ErrorKind::Io,
            ErrorKind::Other,
        ];

        for kind in kinds {
            let diag = Diagnostic::new(kind, format!("{} error occurred", kind), Span::new(0, 2));
            let output = render(&diag, &map);
            assert!(output.contains(&format!("{} error occurred", kind)));
            assert!(output.contains("test2.soma"));
            assert!(output.contains("1:1")); // The span 0..2 starts at line 1 col 1
        }
    }
}
