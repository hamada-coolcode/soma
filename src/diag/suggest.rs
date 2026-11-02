//! "did you mean" and code help text (owned by task 23).

fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let mut dp = vec![vec![0; b_chars.len() + 1]; a_chars.len() + 1];

    for i in 0..=a_chars.len() {
        dp[i][0] = i;
    }
    for j in 0..=b_chars.len() {
        dp[0][j] = j;
    }

    for i in 1..=a_chars.len() {
        for j in 1..=b_chars.len() {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[a_chars.len()][b_chars.len()]
}

/// Closest candidate within edit distance 2, ties broken lexicographically.
pub fn did_you_mean(name: &str, candidates: &[String]) -> Option<String> {
    if name.is_empty() || candidates.is_empty() {
        return None;
    }

    let max_dist = if name.chars().count() > 3 { 2 } else { 1 };

    let mut best_dist = usize::MAX;
    let mut best_candidates = Vec::new();

    for cand in candidates {
        let dist = levenshtein(name, cand);
        if dist <= max_dist {
            if dist < best_dist {
                best_dist = dist;
                best_candidates.clear();
                best_candidates.push(cand);
            } else if dist == best_dist {
                best_candidates.push(cand);
            }
        }
    }

    if !best_candidates.is_empty() {
        best_candidates.sort();
        return Some(best_candidates[0].clone());
    }

    let name_lower = name.to_lowercase();
    let mut best_dist_ci = usize::MAX;
    let mut best_candidates_ci = Vec::new();

    for cand in candidates {
        let cand_lower = cand.to_lowercase();
        let dist = levenshtein(&name_lower, &cand_lower);
        if dist <= max_dist {
            if dist < best_dist_ci {
                best_dist_ci = dist;
                best_candidates_ci.clear();
                best_candidates_ci.push(cand);
            } else if dist == best_dist_ci {
                best_candidates_ci.push(cand);
            }
        }
    }

    if !best_candidates_ci.is_empty() {
        best_candidates_ci.sort();
        return Some(best_candidates_ci[0].clone());
    }

    None
}

/// Long-form help for a diagnostic code such as `"E0301"`.
pub fn suggest_help(code: &str) -> Option<&'static str> {
    match code {
        "E0301" => Some("Cannot assign to a read-only parameter."),
        "E0302" => Some("Name is undefined in this scope."),
        "E0303" => Some("Duplicate definition in the same scope."),
        "E0304" => Some("Invalid use of a name."),
        "E0400" => Some("Type error or mismatch."),
        "E0500" => Some("General type check error."),
        "E0501" => Some("Method not found."),
        "E0502" => Some("Field not found."),
        "E0503" => Some("Type mismatch in assignment."),
        "E0504" => Some("Invalid number of arguments."),
        "E0505" => Some("Trait not implemented."),
        "E0701" => Some("Unreachable code."),
        "E0702" => Some("Missing return statement."),
        "W0101" => Some("Unused variable."),
        "W0102" => Some("Unused function."),
        "W0103" => Some("Unused parameter."),
        _ => None,
    }
}
