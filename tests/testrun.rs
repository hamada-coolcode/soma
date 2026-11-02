use soma::testrun::run_tests;

#[test]
fn test_run_tests() {
    let src = r#"
test
  "pass 1"
    x = 1
    x

test
  "fail 1"
    undefined_name

test
  "pass 2"
    x = 10
    x
"#;
    let res = run_tests("test", src).expect("should not return load error");
    assert_eq!(res.passed, 2);
    assert_eq!(res.failed, 1);
    assert_eq!(
        res.output,
        "test \"pass 1\" ... ok\ntest \"fail 1\" ... FAILED (undefined name: undefined_name)\ntest \"pass 2\" ... ok\n"
    );
}

#[test]
fn test_no_tests() {
    let src = "const\n  X = 1\n";
    let res = run_tests("test", src);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().message, "no tests found");
}
