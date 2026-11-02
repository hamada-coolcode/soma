use soma::codegen::emit_c_checked;
use soma::driver;
use soma::desugar;
use std::process::Command;
use std::fs;

#[test]
fn test_hello_world() {
    let src = "fn\n  main -> !\n    output(\"Hello, world!\")";
    let parsed = driver::parse_source("test", src).unwrap();
    let items = desugar::lower(parsed.items);
    let c = emit_c_checked(&items).unwrap();
    assert!(c.contains("int main(void)"));
    assert!(c.contains("Hello, world!"));
}

#[test]
fn test_arithmetic() {
    let src = "fn\n  add a: int b: int -> int\n    return a + b";
    let parsed = driver::parse_source("test", src).unwrap();
    let items = desugar::lower(parsed.items);
    let c = emit_c_checked(&items).unwrap();
    assert!(c.contains("long long add(long long a, long long b)"));
    assert!(c.contains("return (a + b);"));
}

#[test]
fn test_object() {
    let src = "object\n  Point\n    x: int\n    y: int\n\n    sum -> int\n      return self.x + self.y";
    let parsed = driver::parse_source("test", src).unwrap();
    let items = desugar::lower(parsed.items);
    let c = emit_c_checked(&items).unwrap();
    assert!(c.contains("typedef struct Point Point;"));
    assert!(c.contains("struct Point {"));
    assert!(c.contains("long long x;"));
    assert!(c.contains("long long y;"));
}

#[test]
fn test_compile_run() {
    if Command::new("which").arg("cc").output().is_err() {
        return;
    }

    // tests/golden/hello.soma
    let hello_src = fs::read_to_string("tests/golden/hello.soma").unwrap();
    let parsed = driver::parse_source("hello.soma", &hello_src).unwrap();
    let items = desugar::lower(parsed.items);
    let c_code = emit_c_checked(&items).unwrap();

    let tmp_dir = std::env::temp_dir();
    let hello_c = tmp_dir.join("hello.c");
    let hello_bin = tmp_dir.join("hello");
    fs::write(&hello_c, c_code).unwrap();

    let cc_status = Command::new("cc")
        .arg("-std=c11")
        .arg("-O2")
        .arg("-w")
        .arg(&hello_c)
        .arg("-o")
        .arg(&hello_bin)
        .status()
        .unwrap();

    assert!(cc_status.success());
    let run_output = Command::new(&hello_bin).output().unwrap();
    assert_eq!(String::from_utf8(run_output.stdout).unwrap(), "Hello, world!\n");

    // tests/golden/arith.soma
    let arith_src_with_main = "fn\n  add a: int b: int -> int\n    return a + b\n  main -> !\n    output(\"3\")";
    let parsed = driver::parse_source("arith.soma", &arith_src_with_main).unwrap();
    let items = desugar::lower(parsed.items);
    let c_code = emit_c_checked(&items).unwrap();

    let arith_c = tmp_dir.join("arith.c");
    let arith_bin = tmp_dir.join("arith");
    fs::write(&arith_c, c_code).unwrap();

    let cc_status = Command::new("cc")
        .arg("-std=c11")
        .arg("-O2")
        .arg("-w")
        .arg(&arith_c)
        .arg("-o")
        .arg(&arith_bin)
        .status()
        .unwrap();

    assert!(cc_status.success());
    let run_output = Command::new(&arith_bin).output().unwrap();
    assert_eq!(String::from_utf8(run_output.stdout).unwrap(), "3\n");
}
