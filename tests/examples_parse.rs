//! Every reference program must at least lex and parse. This is the first real
//! end-to-end check of the merged lexer + parser lanes.

use std::fs;
use std::path::Path;

fn soma_files() -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut dirs: Vec<_> = fs::read_dir(examples)
        .expect("examples/ missing")
        .map(|e| e.unwrap().path())
        .collect();
    dirs.sort();
    for dir in dirs {
        let f = dir.join("main.soma");
        if f.exists() {
            out.push(f);
        }
    }
    out
}

#[test]
fn every_example_lexes_and_parses() {
    let files = soma_files();
    assert!(
        files.len() >= 14,
        "expected 14 examples, found {}",
        files.len()
    );

    let mut failures = Vec::new();
    for f in &files {
        let src = fs::read_to_string(f).unwrap();
        match soma::lexer::scan(&src) {
            Err(e) => failures.push(format!(
                "{}: lex error at byte {}: {}",
                f.display(),
                e.span.start,
                e.message
            )),
            Ok(tokens) => {
                if let Err(e) = soma::parse::parse_file(&tokens) {
                    failures.push(format!(
                        "{}: parse error at byte {}: {}",
                        f.display(),
                        e.span.start,
                        e.message
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} examples failed:\n{}",
        failures.len(),
        files.len(),
        failures.join("\n")
    );
}

#[test]
fn every_example_produces_items() {
    for f in soma_files() {
        let src = fs::read_to_string(&f).unwrap();
        let tokens = soma::lexer::scan(&src).expect("lex");
        let items = soma::parse::parse_file(&tokens).expect("parse");
        assert!(!items.is_empty(), "{} produced no items", f.display());
    }
}
