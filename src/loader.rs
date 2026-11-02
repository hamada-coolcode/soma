//! Module loading for `import` trees (owned by task 17).

use crate::ast::item::{Item, ItemKind, ImportDef};
use crate::diag::{Diagnostic, ErrorKind, Span};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct Loaded {
    pub files: Vec<LoadedFile>,
}

#[derive(Debug)]
pub struct LoadedFile {
    pub path: String,
    pub source: String,
}

struct LoaderCtx {
    loaded: Loaded,
    visited: HashSet<PathBuf>,
    in_progress: Vec<PathBuf>,
}

impl LoaderCtx {
    fn new() -> Self {
        Self {
            loaded: Loaded::default(),
            visited: HashSet::new(),
            in_progress: Vec::new(),
        }
    }

    fn load(&mut self, path: &Path, span: Span) -> Result<(), Diagnostic> {
        let canon = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

        if self.visited.contains(&canon) {
            return Ok(()); // Already loaded
        }

        if let Some(cycle_start) = self.in_progress.iter().position(|p| p == &canon) {
            let mut cycle = self.in_progress[cycle_start..].to_vec();
            cycle.push(canon);
            let cycle_str = cycle
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(" -> ");
            return Err(Diagnostic::new(
                ErrorKind::Other,
                "E0702",
                span,
            )
            .with_help(format!("cycle: {}", cycle_str)));
        }

        let src = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_e) => {
                let diag = Diagnostic::new(
                    ErrorKind::Other,
                    "E0701",
                    span,
                )
                .with_help(format!("searched path: {}", path.display()));
                return Err(diag);
            }
        };

        self.in_progress.push(canon.clone());

        let toks = crate::lexer::scan(&src)
            .map_err(|e| Diagnostic::new(ErrorKind::Lex, e.message, e.span))?;
        let items = crate::parse::parse_file(&toks).map_err(|e| {
            let d = Diagnostic::new(ErrorKind::Parse, e.message, e.span);
            if e.expected.is_empty() {
                d
            } else {
                d.with_help(format!("expected: {}", e.expected.join(", ")))
            }
        })?;

        let mut imports = Vec::new();
        for item in &items {
            self.collect_imports(item, &mut imports);
        }

        let mut parent_dir = path.parent().unwrap_or_else(|| Path::new("")).to_path_buf();
        if parent_dir.as_os_str().is_empty() {
            parent_dir = PathBuf::from(".");
        }

        for (import_path, import_span) in imports {
            let mut resolved = PathBuf::new();
            if import_path.starts_with("lib") {
                let mut up = parent_dir.clone();
                let mut found = false;
                loop {
                    let maybe = up.join(&import_path);
                    if maybe.exists() {
                        resolved = maybe;
                        found = true;
                        break;
                    }
                    let p = up.parent();
                    if let Some(parent) = p {
                        if parent.as_os_str().is_empty() {
                            up = PathBuf::from(".");
                        } else {
                            up = parent.to_path_buf();
                        }
                    } else {
                        break;
                    }
                    if up.as_os_str() == "." {
                        let maybe = up.join(&import_path);
                        if maybe.exists() {
                            resolved = maybe;
                            found = true;
                        }
                        break;
                    }
                }
                if !found {
                    resolved = parent_dir.join(&import_path);
                }
            } else {
                resolved = parent_dir.join(&import_path);
            }

            self.load(&resolved, import_span)?;
        }

        self.in_progress.pop();
        self.visited.insert(canon);
        self.loaded.files.push(LoadedFile {
            path: path.display().to_string(),
            source: src,
        });

        Ok(())
    }

    fn collect_imports(&self, item: &Item, imports: &mut Vec<(PathBuf, Span)>) {
        if let ItemKind::Import(def) = &item.kind {
            self.collect_import_def(def, PathBuf::new(), imports, item.span);
        }
    }

    fn collect_import_def(&self, def: &ImportDef, base_path: PathBuf, imports: &mut Vec<(PathBuf, Span)>, span: Span) {
        let mut current_path = base_path;
        for seg in &def.path {
            let parts: Vec<&str> = seg.text.split(':').collect();
            for p in parts {
                current_path.push(p);
            }
        }

        if def.children.is_empty() {
            let mut pb = current_path;
            pb.set_extension("soma");
            imports.push((pb, span));
        } else {
            for child in &def.children {
                if let ItemKind::Import(child_def) = &child.kind {
                    self.collect_import_def(child_def, current_path.clone(), imports, child.span);
                }
            }
        }
    }
}

pub fn load_entry(path: &str) -> Result<Loaded, Diagnostic> {
    let mut ctx = LoaderCtx::new();
    ctx.load(Path::new(path), Span::default())?;
    Ok(ctx.loaded)
}
