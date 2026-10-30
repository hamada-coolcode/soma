use soma::emit_json::emit_json;
use std::fs;

fn parse_file(path: &str) -> Vec<soma::ast::item::Item> {
    let src = fs::read_to_string(path).unwrap();
    soma::driver::parse_source(path, &src).unwrap().items
}

fn validate_json(s: &str) -> Result<(), String> {
    let mut brackets = Vec::new();
    let mut in_str = false;
    let mut escape = false;

    for c in s.chars() {
        if in_str {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_str = false;
            }
        } else {
            match c {
                '"' => in_str = true,
                '{' | '[' => brackets.push(c),
                '}' => {
                    if brackets.pop() != Some('{') {
                        return Err("Unbalanced }".to_string());
                    }
                }
                ']' => {
                    if brackets.pop() != Some('[') {
                        return Err("Unbalanced ]".to_string());
                    }
                }
                _ => {}
            }
        }
    }
    if in_str {
        return Err("Unclosed string".to_string());
    }
    if !brackets.is_empty() {
        return Err("Unclosed brackets".to_string());
    }
    Ok(())
}

#[test]
fn test_golden_arith() {
    let items = parse_file("tests/golden/arith.soma");
    let json = emit_json(&items);
    validate_json(&json).unwrap();
    assert!(json.contains("\"Fn\""));
    assert!(json.contains("\"add\""));
}

#[test]
fn test_examples_parse_and_validate() {
    for entry in fs::read_dir("examples").unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            let path = entry.path().join("main.soma");
            if path.exists() {
                let items = parse_file(path.to_str().unwrap());
                let json = emit_json(&items);
                if let Err(e) = validate_json(&json) {
                    panic!("Invalid JSON for {}: {}", path.display(), e);
                }
            }
        }
    }
}

#[test]
fn test_escapes() {
    let s = soma::emit_json::emit_json(&[]);
    assert_eq!(s, "[]\n");
}

#[test]
fn test_unknown_kinds() {}

#[test]
fn test_string_escapes_advanced() {
    let span = soma::diag::Span::new(0, 0);
    // Construct some dummy parsed string items
    let s = "\"\n\t\r\\\x19";
    let expr = soma::ast::expr::Expr {
        kind: soma::ast::expr::ExprKind::Str(s.to_string()),
        span,
    };
    let items = vec![soma::ast::item::Item {
        attrs: vec![],
        kind: soma::ast::item::ItemKind::Const(soma::ast::item::ConstDef {
            name: soma::ast::item::Name::new("A", span),
            value: expr,
        }),
        span,
    }];
    let json = soma::emit_json::emit_json(&items);
    assert!(json.contains("\\\"\\n\\t\\r\\\\\\u0019"));
}
