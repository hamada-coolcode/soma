use soma::loader::load_entry;
use std::fs;
use std::path::Path;

fn setup_fixtures(test_name: &str) {
    let dir_name = format!("tests/loader_fixtures_{}", test_name);
    let dir = Path::new(&dir_name);
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }
    fs::create_dir_all(&dir).unwrap();

    // chain
    if test_name == "chain" {
        fs::create_dir_all(dir.join("lib")).unwrap();
        fs::write(dir.join("a.soma"), "import\n  lib:b\n").unwrap();
        fs::write(dir.join("lib/b.soma"), "import\n  lib:c\n").unwrap();
        fs::write(dir.join("lib/c.soma"), "fn\n  c\n    return\n").unwrap();
    }

    // duplicate
    if test_name == "duplicate" {
        fs::create_dir_all(dir.join("lib")).unwrap();
        fs::write(dir.join("a.soma"), "import\n  lib:b\n  lib:c\n").unwrap();
        fs::write(dir.join("lib/b.soma"), "import\n  lib:c\n").unwrap();
        fs::write(dir.join("lib/c.soma"), "fn\n  c\n    return\n").unwrap();
    }

    // missing
    if test_name == "missing" {
        fs::create_dir_all(dir.join("lib")).unwrap();
        fs::write(dir.join("a.soma"), "import\n  lib:d\n").unwrap();
    }

    // cycle
    if test_name == "cycle" {
        fs::create_dir_all(dir.join("lib")).unwrap();
        fs::write(dir.join("a.soma"), "import\n  lib:b\n").unwrap();
        fs::write(dir.join("lib/b.soma"), "import\n  lib:c\n").unwrap();
        fs::write(dir.join("lib/c.soma"), "import\n  lib:b\n").unwrap();
    }

    // dir run
    if test_name == "dir_run" {
        fs::create_dir_all(dir.join("dir_run")).unwrap();
        fs::write(dir.join("dir_run/ok1.soma"), "fn\n  ok\n    return\n").unwrap();
        fs::write(dir.join("dir_run/ok2.soma"), "fn\n  ok\n    return\n").unwrap();
        fs::write(dir.join("dir_run/err.soma"), "fn err) (\n").unwrap();
    }
}

#[test]
fn test_chain() {
    setup_fixtures("chain");
    let loaded = load_entry("tests/loader_fixtures_chain/a.soma").unwrap();
    assert_eq!(loaded.files.len(), 3);
}

#[test]
fn test_duplicate() {
    setup_fixtures("duplicate");
    let loaded = load_entry("tests/loader_fixtures_duplicate/a.soma").unwrap();
    assert_eq!(loaded.files.len(), 3); // a, b, c (c should not be loaded twice)
}

#[test]
fn test_missing() {
    setup_fixtures("missing");
    let err = load_entry("tests/loader_fixtures_missing/a.soma").unwrap_err();
    assert_eq!(err.message, "E0701");
}

#[test]
fn test_cycle() {
    setup_fixtures("cycle");
    let err = load_entry("tests/loader_fixtures_cycle/a.soma").unwrap_err();
    assert_eq!(err.message, "E0702");
}
