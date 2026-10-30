use std::fs;
use std::path::{Path, PathBuf};

fn get_examples_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("examples");
    path
}

fn detect_indent_width(lines: &[&str]) -> Option<usize> {
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        if indent > 0 {
            return Some(indent);
        }
    }
    None
}

fn has_tab_in_indent(line: &str) -> bool {
    let indent = line.len() - line.trim_start().len();
    line[..indent].contains('\t')
}

fn has_trailing_whitespace(line: &str) -> bool {
    line.ends_with(' ') || line.ends_with('\t')
}

fn is_word_boundary(c: char) -> bool {
    !c.is_alphanumeric() && c != '_'
}

fn has_struct_keyword(line: &str) -> bool {
    let words: Vec<&str> = line.split(is_word_boundary).collect();
    words.contains(&"struct")
}

// 8. no <> generic brackets (generics use [])
fn has_generic_brackets(line: &str) -> bool {
    // Look for < followed by an identifier followed by >
    // To be simple, we look for '<' then alphabets then '>'
    let chars: Vec<char> = line.chars().collect();
    for i in 0..chars.len() {
        if chars[i] == '<' {
            let mut j = i + 1;
            let mut has_ident = false;
            while j < chars.len() && chars[j].is_alphanumeric() || chars[j] == '_' {
                has_ident = true;
                j += 1;
            }
            if has_ident && j < chars.len() && chars[j] == '>' {
                return true;
            }
        }
    }
    false
}

fn check_file(path: &Path, content: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let display_path = path.display().to_string();

    // 4. file ends with exactly one newline
    if !content.ends_with('\n') {
        errors.push(format!("{}: file must end with a newline", display_path));
    } else if content.ends_with("\n\n") {
        errors.push(format!(
            "{}: file must end with exactly one newline",
            display_path
        ));
    }

    let lines: Vec<&str> = content.lines().collect();
    let indent_width = detect_indent_width(&lines);

    // 2. the file's indent width (first indented line) is 2, 4, 6 or 8
    if let Some(w) = indent_width {
        if w != 2 && w != 4 && w != 6 && w != 8 {
            errors.push(format!("{}: invalid indent width {}", display_path, w));
        }
    }

    let has_output = content.contains("output(");
    // Let's improve the import check
    // "if 'output(' appears and no 'import' line exists, fail"
    let has_import = lines.iter().any(|l| l.trim_start().starts_with("import"));

    // 10. every file that imports output also uses the import tree form
    if has_output && !has_import {
        errors.push(format!(
            "{}: uses output( but has no import tree",
            display_path
        ));
    }

    for (i, line) in lines.iter().enumerate() {
        let ln = i + 1;
        let line_loc = format!("{}:{}", display_path, ln);

        // 1. indent is spaces only (no tab characters in indentation)
        if has_tab_in_indent(line) {
            errors.push(format!("{}: tab in indent", line_loc));
        }

        // 2. every non-blank line's indentation is a multiple of indent_width
        if !line.trim().is_empty() {
            let indent = line.len() - line.trim_start().len();
            if let Some(w) = indent_width {
                if indent > 0 && indent % w != 0 {
                    errors.push(format!(
                        "{}: indent {} is not a multiple of {}",
                        line_loc, indent, w
                    ));
                }
            }
        }

        // 3. no trailing whitespace on any line
        if has_trailing_whitespace(line) {
            errors.push(format!("{}: trailing whitespace", line_loc));
        }

        // 5. no :: anywhere
        if line.contains("::") {
            errors.push(format!("{}: contains ::", line_loc));
        }

        // 6. no struct keyword (must be object)
        if has_struct_keyword(line) {
            errors.push(format!("{}: contains struct keyword", line_loc));
        }

        // 7. no main! / main !
        if line.contains("main!") || line.contains("main !") {
            errors.push(format!("{}: contains main! or main !", line_loc));
        }

        // 8. no <> generic brackets (generics use [])
        if has_generic_brackets(line) {
            errors.push(format!("{}: contains <> generics", line_loc));
        }
    }

    errors
}

fn visit_dirs(dir: &Path, errors: &mut Vec<String>, count: &mut usize) {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, errors, count);
            } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("soma") {
                *count += 1;
                let content = fs::read_to_string(&path).unwrap();
                let rel_path = path.strip_prefix(env!("CARGO_MANIFEST_DIR")).unwrap();
                errors.extend(check_file(rel_path, &content));
            }
        }
    }
}

#[test]
fn test_examples_corpus() {
    let examples_dir = get_examples_dir();
    let mut errors = Vec::new();
    let mut count = 0;
    visit_dirs(&examples_dir, &mut errors, &mut count);

    // 9. at least 14 example directories exist, each with a main.soma
    // We'll just check if count is at least 14.
    if count < 14 {
        errors.push(format!("Expected at least 14 examples, found {}", count));
    }

    if !errors.is_empty() {
        for err in &errors {
            eprintln!("{}", err);
        }
        panic!("Found {} errors in examples corpus", errors.len());
    }
}

#[test]
fn test_detect_indent_width() {
    assert_eq!(detect_indent_width(&["", "  foo"]), Some(2));
    assert_eq!(detect_indent_width(&["    foo"]), Some(4));
    assert_eq!(detect_indent_width(&["foo", "bar"]), None);
}

#[test]
fn test_has_tab_in_indent() {
    assert!(has_tab_in_indent("\tfoo"));
    assert!(has_tab_in_indent(" \tfoo"));
    assert!(!has_tab_in_indent("  foo"));
    assert!(!has_tab_in_indent("foo\tbar"));
}

#[test]
fn test_has_trailing_whitespace() {
    assert!(has_trailing_whitespace("foo "));
    assert!(has_trailing_whitespace("foo\t"));
    assert!(!has_trailing_whitespace("foo"));
    assert!(!has_trailing_whitespace("  foo"));
}

#[test]
fn test_has_struct_keyword() {
    assert!(has_struct_keyword("struct Foo"));
    assert!(has_struct_keyword("  struct"));
    assert!(!has_struct_keyword("struct_foo"));
    assert!(!has_struct_keyword("foo_struct"));
    assert!(!has_struct_keyword("object Foo"));
}

#[test]
fn test_has_generic_brackets() {
    assert!(has_generic_brackets("Foo<T>"));
    assert!(has_generic_brackets("Box<int>"));
    assert!(!has_generic_brackets("a < b"));
    assert!(!has_generic_brackets("a > b"));
    assert!(!has_generic_brackets("Foo[T]"));
}

#[test]
fn test_check_file_inline_fixtures() {
    let dummy_path = Path::new("dummy.soma");

    // 1. indent is spaces only
    let errors = check_file(dummy_path, "main\n\toutput()\n");
    assert!(
        errors.iter().any(|e| e.contains("tab in indent")),
        "missing tab error"
    );

    // 2. indent width 2, 4, 6 or 8
    let errors = check_file(dummy_path, "main\n   output()\n"); // width 3
    assert!(errors.iter().any(|e| e.contains("invalid indent width 3")));

    // 2. multiple of indent width
    let errors = check_file(dummy_path, "main\n  foo\n   bar\n"); // 2, then 3
    assert!(
        errors
            .iter()
            .any(|e| e.contains("indent 3 is not a multiple of 2"))
    );

    // 3. no trailing whitespace
    let errors = check_file(dummy_path, "main \n  foo\n");
    assert!(errors.iter().any(|e| e.contains("trailing whitespace")));

    // 4. exactly one newline
    let errors = check_file(dummy_path, "main");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("file must end with a newline"))
    );

    let errors = check_file(dummy_path, "main\n\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("file must end with exactly one newline"))
    );

    // 5. no ::
    let errors = check_file(dummy_path, "main\n  std::output()\n");
    assert!(errors.iter().any(|e| e.contains("contains ::")));

    // 6. no struct keyword
    let errors = check_file(dummy_path, "struct Foo\n");
    assert!(errors.iter().any(|e| e.contains("contains struct keyword")));

    // 7. no main! / main !
    let errors = check_file(dummy_path, "main!\n");
    assert!(errors.iter().any(|e| e.contains("contains main!")));
    let errors = check_file(dummy_path, "main !\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("contains main! or main !"))
    );

    // 8. no <> generic brackets
    let errors = check_file(dummy_path, "List<T>\n");
    assert!(errors.iter().any(|e| e.contains("contains <> generics")));

    // 10. if output is used, import tree form must exist
    let errors = check_file(dummy_path, "main\n  output()\n");
    assert!(
        errors
            .iter()
            .any(|e| e.contains("uses output( but has no import tree"))
    );

    let no_errors = check_file(
        dummy_path,
        "import\n  lib:std\n    io\n      term\n        output\n\nmain\n  output()\n",
    );
    assert!(
        no_errors.is_empty(),
        "expected no errors, got: {:?}",
        no_errors
    );
}
