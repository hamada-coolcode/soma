use soma::diag::suggest::{did_you_mean, suggest_help};

#[test]
fn test_did_you_mean() {
    let cases = vec![
        // undefind_name -> undefined_name
        ("undefind_name", vec!["undefined_name"], Some("undefined_name")),
        // reslts -> results
        ("reslts", vec!["results"], Some("results")),
        // exact match
        ("results", vec!["results"], Some("results")),
        // beyond distance 2
        ("foo", vec!["foobar"], None),
        ("abcdef", vec!["ab"], None),
        // short-name behaviour (dist <= 1)
        ("abc", vec!["abx"], Some("abx")), // dist 1
        ("abc", vec!["axy"], None), // dist 2 for len 3 -> None
        // tie-break (lexicographical)
        ("car", vec!["bar", "cat"], Some("bar")), // dist 1 for both, bar < cat
        // case-insensitive fallback
        ("FOOBAR", vec!["foobar"], Some("foobar")), // dist 6 case-sens, dist 0 case-insens
        ("Foobar", vec!["foobar"], Some("foobar")), // dist 1 case-sens (allowed as len 6 > 3)
        // empty candidate slice
        ("name", vec![], None),
        // empty name
        ("", vec!["name"], None),
        // empty both
        ("", vec![], None),
        // multiple same dist case-insensitive, ties broken lexicographically
        ("CaR", vec!["bar", "cat"], Some("bar")),
    ];

    for (name, candidates, expected) in cases {
        let candidates_vec: Vec<String> = candidates.into_iter().map(|s| s.to_string()).collect();
        let result = did_you_mean(name, &candidates_vec);
        assert_eq!(result.as_deref(), expected, "failed for name: {}", name);
    }
}

#[test]
fn test_suggest_help() {
    let codes = vec![
        "E0301", "E0302", "E0303", "E0304",
        "E0500", "E0501", "E0502", "E0503", "E0504", "E0505",
        "E0701", "E0702",
        "W0101", "W0102", "W0103",
        "E0400"
    ];

    for code in codes {
        assert!(suggest_help(code).is_some(), "missing help for code: {}", code);
    }

    assert_eq!(suggest_help("GARBAGE"), None);
    assert_eq!(suggest_help("E9999"), None);
}
