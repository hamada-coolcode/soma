//! "did you mean" and code help text (owned by task 23).

/// Closest candidate within edit distance 2, ties broken lexicographically.
pub fn did_you_mean(_name: &str, _candidates: &[String]) -> Option<String> {
    None
}

/// Long-form help for a diagnostic code such as `"E0301"`.
pub fn suggest_help(_code: &str) -> Option<&'static str> {
    None
}
