use std::fs;
use std::path::Path;

#[test]
fn test_golden_files() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let golden_dir = manifest_dir.join("tests").join("golden");
    let names = ["hello", "arith", "object"];

    for name in names {
        let soma_path = golden_dir.join(format!("{}.soma", name));
        let psoma_path = golden_dir.join(format!("{}.psoma", name));

        let src = fs::read_to_string(&soma_path).unwrap();
        let expected_psoma = fs::read_to_string(&psoma_path).unwrap();

        // Go through the real driver path (parse -> lower -> emit) so the
        // goldens cover the pipeline the CLI actually prints.
        let parsed = soma::driver::parse_source(&format!("{name}.soma"), &src).unwrap();
        let actual_psoma = soma::driver::emit_psoma(&parsed);

        assert_eq!(
            actual_psoma, expected_psoma,
            "Mismatch in golden file {}",
            name
        );
    }
}

#[test]
fn test_examples_output_nonempty() {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut dirs: Vec<_> = fs::read_dir(examples)
        .expect("examples/ missing")
        .map(|e| e.unwrap().path())
        .collect();
    dirs.sort();

    let mut count = 0;
    for dir in dirs {
        let f = dir.join("main.soma");
        if f.exists() {
            count += 1;
            let src = fs::read_to_string(&f).unwrap();
            let tokens = soma::lexer::scan(&src).unwrap();
            let items = soma::parse::parse_file(&tokens).unwrap();
            let actual_psoma = soma::desugar::psoma::emit(&items);

            assert!(!actual_psoma.is_empty(), "Empty output for {}", f.display());
            assert!(actual_psoma.contains("fn "), "No fn found in output for {}", f.display());
        }
    }
    assert!(count >= 14, "Expected at least 14 examples");
}

#[test]
fn test_determinism() {
    let src = "fn\n  add a: int b: int -> int\n    return a + b";
    let tokens = soma::lexer::scan(src).unwrap();
    let items = soma::parse::parse_file(&tokens).unwrap();

    let a = soma::desugar::psoma::emit(&items);
    let b = soma::desugar::psoma::emit(&items);
    assert_eq!(a, b);
}
