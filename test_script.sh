#!/bin/bash
cat > src/testrun.rs << 'RS_EOF'
//! `soma test` — execute `test "…"` blocks (owned by task 24).

use crate::diag::{Diagnostic, ErrorKind, Span};
use crate::driver;
use crate::desugar;
use crate::interp::eval::Evaluator;
use crate::ast::item::ItemKind;

#[derive(Debug, Default)]
pub struct TestReport {
    pub passed: usize,
    pub failed: usize,
    pub output: String,
}

pub fn run_tests(name: &str, src: &str) -> Result<TestReport, Diagnostic> {
    let parsed = driver::parse_source(name, src)?;
    let items = desugar::lower(parsed.items);

    let mut evaluator = Evaluator::new();
    evaluator.run_items(&items)?;

    let tests: Vec<_> = items.into_iter().filter_map(|item| {
        if let ItemKind::Test(test_def) = item.kind {
            Some(test_def)
        } else {
            None
        }
    }).collect();

    if tests.is_empty() {
        return Err(Diagnostic::new(
            ErrorKind::Other,
            "no tests found",
            Span::default(),
        ));
    }

    let mut report = TestReport::default();

    for test in tests {
        match evaluator.eval_block(&test.body) {
            Ok(_) => {
                report.passed += 1;
                report.output.push_str(&format!("test \"{}\" ... ok\n", test.name));
            }
            Err(e) => {
                report.failed += 1;
                report.output.push_str(&format!("test \"{}\" ... FAILED ({})\n", test.name, e.message));
            }
        }
    }

    Ok(report)
}
RS_EOF
