# Task 24 — `soma test`

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/testrun.rs`
- `tests/testrun.rs`

## Goal
Run `test "name"` blocks and report results.

- `run_tests(name, src) -> Result<TestReport, Diagnostic>` parses the file
  (via `driver`/`parse`, whichever is public), collects every `test` block in
  source order, and evaluates each body with the interpreter;
- each test prints `test "<name>" ... ok` or `... FAILED (<message>)`;
- `TestReport { passed, failed, output }`; `output` holds the whole transcript;
- a runtime error inside a test is a **test failure**, not a load error;
- no tests in the file → a load error with a clear message.

## Acceptance (definition of done)
- `tests/testrun.rs`: fixture with 2 passing + 1 failing test (assert
  `passed == 2`, `failed == 1`, transcript order); a file whose test reads an
  undefined name fails that test only; empty file errors cleanly.
- No panics when the interpreter is still incomplete: unimplemented constructs
  surface as a test failure message.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`,
  `design/`, `examples/`, `jules/`, or `tests/golden/`.
- CLI wiring (`soma test`) is done by the integrator — library only.
- Keep the existing `pub fn run_tests` signature. No new dependencies.
- `cargo test` must pass. No `todo!()`/`unimplemented!()`.
