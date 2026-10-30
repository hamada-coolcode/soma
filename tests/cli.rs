use std::process::Command;

fn get_bin() -> &'static str {
    env!("CARGO_BIN_EXE_soma")
}

#[test]
fn test_version() {
    let output = Command::new(get_bin())
        .arg("--version")
        .output()
        .expect("Failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("soma"));
}

#[test]
fn test_parse_ok() {
    let output = Command::new(get_bin())
        .args(["parse", "examples/hello-world/main.soma"])
        .output()
        .expect("Failed to execute process");

    assert!(output.status.success(), "stdout: {}, stderr: {}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_parse_emit_psoma() {
    let output = Command::new(get_bin())
        .args(["parse", "examples/hello-world/main.soma", "--emit-psoma"])
        .output()
        .expect("Failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.is_empty());
}

#[test]
fn test_parse_missing_file() {
    let output = Command::new(get_bin())
        .args(["parse", "missing_file.soma"])
        .output()
        .expect("Failed to execute process");

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_parse_syntax_error() {
    let err_file = "tests/temp_syntax_error.soma";
    std::fs::write(err_file, "fn main() { let x = ; }").unwrap();

    // ensure it is cleaned up on panic
    struct Cleanup;
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_file("tests/temp_syntax_error.soma");
        }
    }
    let _cleanup = Cleanup;

    let output = Command::new(get_bin())
        .args(["parse", err_file])
        .output()
        .expect("Failed to execute process");

    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_check_ok() {
    let output = Command::new(get_bin())
        .args(["check", "examples/hello-world/main.soma"])
        .output()
        .expect("Failed to execute process");

    assert!(output.status.success(), "stdout: {}, stderr: {}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
}
