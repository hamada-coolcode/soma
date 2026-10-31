use soma::ast::item::{Item, ItemKind};
use soma::driver::parse_source;
use std::fs;
use std::path::Path;

fn check_spans(items: &[Item]) {
    for item in items {
        assert!(
            item.span.start <= item.span.end,
            "Invalid span bounds: start {} > end {}",
            item.span.start,
            item.span.end
        );
        assert!(
            item.span.end > item.span.start,
            "Empty span: start {} == end {}",
            item.span.start,
            item.span.end
        );

        for attr in &item.attrs {
            assert!(attr.name.span.start < attr.name.span.end);
        }

        match &item.kind {
            ItemKind::Fn(f) => {
                assert!(!f.body.stmts.is_empty() || f.body.span.end > f.body.span.start);
            }
            ItemKind::Object(o) => {
                for method in &o.methods {
                    assert!(
                        !method.body.stmts.is_empty()
                            || method.body.span.end > method.body.span.start
                    );
                }
            }
            ItemKind::Test(t) => {
                assert!(!t.body.stmts.is_empty() || t.body.span.end > t.body.span.start);
            }
            _ => {}
        }
    }
}

fn check_identifiers(items: &[Item]) {
    use std::collections::HashSet;
    let mut names = HashSet::new();
    for item in items {
        let name = match &item.kind {
            ItemKind::Fn(f) => f.name.text.clone(),
            ItemKind::Object(o) => o.name.text.clone(),
            ItemKind::Trait(t) => t.name.text.clone(),
            ItemKind::Error(e) => e.name.text.clone(),
            ItemKind::Enum(e) => e.name.text.clone(),
            ItemKind::Const(c) => c.name.text.clone(),
            ItemKind::Test(t) => t.name.clone(),
            ItemKind::Import(_) => continue, // Handled differently if imports bind names
        };
        assert!(
            names.insert(name.clone()),
            "Duplicate identifier found at top level: {}",
            name
        );

        match &item.kind {
            ItemKind::Object(o) => {
                let mut member_names = HashSet::new();
                for field in &o.fields {
                    assert!(
                        member_names.insert(field.name.text.clone()),
                        "Duplicate field: {}",
                        field.name.text
                    );
                }
                for method in &o.methods {
                    assert!(
                        member_names.insert(method.name.text.clone()),
                        "Duplicate method: {}",
                        method.name.text
                    );
                }
            }
            ItemKind::Enum(e) => {
                let mut variant_names = HashSet::new();
                for v in &e.variants {
                    assert!(
                        variant_names.insert(v.name.text.clone()),
                        "Duplicate variant: {}",
                        v.name.text
                    );
                }
            }
            ItemKind::Error(e) => {
                let mut variant_names = HashSet::new();
                for v in &e.variants {
                    assert!(
                        variant_names.insert(v.text.clone()),
                        "Duplicate error variant: {}",
                        v.text
                    );
                }
            }
            ItemKind::Trait(t) => {
                let mut method_names = HashSet::new();
                for m in &t.methods {
                    assert!(
                        method_names.insert(m.name.text.clone()),
                        "Duplicate trait method: {}",
                        m.name.text
                    );
                }
            }
            _ => {}
        }
    }
}

fn test_file(path: &Path) {
    let src = fs::read_to_string(path).unwrap();
    let name = path.file_name().unwrap().to_str().unwrap();

    let res1 = parse_source(name, &src).unwrap_or_else(|e| {
        panic!(
            "File {} failed to parse at offset {}: {}",
            name, e.span.start, e.message
        );
    });

    let res2 = parse_source(name, &src).unwrap();
    assert_eq!(
        res1.items, res2.items,
        "Parsing is not deterministic for {}",
        name
    );

    check_spans(&res1.items);
    check_identifiers(&res1.items);
}

#[test]
fn test_corpus_files() {
    let mut entries: Vec<_> = fs::read_dir("tests/corpus")
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    assert_eq!(entries.len(), 10, "Expected exactly 10 corpus files");
    for path in entries {
        if path.extension().map_or(false, |ext| ext == "soma") {
            test_file(&path);
        }
    }
}

#[test]
fn test_example_files() {
    let mut entries: Vec<_> = fs::read_dir("examples")
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            let main_path = path.join("main.soma");
            if main_path.exists() {
                test_file(&main_path);
            }
        }
    }
}
