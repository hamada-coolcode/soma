use soma::diag::error::{DiagCode, Diagnostic, ErrorKind};
use soma::diag::{SourceMap, Span};

#[test]
fn test_render_with_codes() {
    let mut map = SourceMap::new();
    map.add("test.soma", "let x = 1;\n");

    let cases = vec![
        (DiagCode::E0100, ErrorKind::Lex),
        (DiagCode::E0200, ErrorKind::Parse),
        (DiagCode::E0301, ErrorKind::Other),
        (DiagCode::E0401, ErrorKind::Type),
        (DiagCode::E0501, ErrorKind::Other),
    ];

    for (code, kind) in cases {
        let diag = Diagnostic::new(kind, "Test error", Span::new(0, 5)).with_code(code);
        let rendered = soma::diag::render::render(&diag, &map);

        let expected = format!("[{}]", code.as_str());
        assert!(
            rendered.contains(&expected),
            "Expected output to contain '{}', but it did not.\nRendered:\n{}",
            expected,
            rendered
        );
    }
}

#[test]
fn test_diagcode_roundtrip() {
    let codes = vec![
        DiagCode::E0100,
        DiagCode::E0200,
        DiagCode::E0300,
        DiagCode::E0301,
        DiagCode::E0302,
        DiagCode::E0303,
        DiagCode::E0400,
        DiagCode::E0401,
        DiagCode::E0402,
        DiagCode::E0403,
        DiagCode::E0500,
        DiagCode::E0501,
    ];

    for code in codes {
        let s = code.as_str();
        let parsed = DiagCode::parse(s);
        assert_eq!(parsed, Some(code), "Failed to round-trip {:?}", code);
    }

    assert_eq!(DiagCode::parse("E9999"), None);
}
