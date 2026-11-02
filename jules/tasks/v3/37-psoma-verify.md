# Task 37 — Verify the frozen `.psoma` contract

Read `jules/README.md`, `jules/psoma.md` and `design/08-pipeline.md` first.

## Files you may touch (ONLY these)
- `src/desugar/psoma.rs`
- `tests/psoma_contract.rs` (new file)

## Goal
Make sure the emitter matches the frozen contract exactly, and that the contract
is enforced by tests, not by eyeballing:
- every rule in `jules/psoma.md` gets a test: stability of output across runs,
  one statement per line, 2-space indentation, no trailing whitespace, `+`
  preserved verbatim (never `.add()`), `propagate(x)` form, comprehension
  lowering shape;
- add `psoma::verify(text: &str) -> Vec<String>` returning contract violations
  (indentation, trailing whitespace, unbalanced braces, unknown call form);
- run `verify` over the emitted output of all 14 examples and assert zero
  violations;
- goldens must still match byte-for-byte.

## Acceptance
- `tests/psoma_contract.rs` green, `tests/psoma_golden.rs` untouched and green.
- No change to emitted output.

## Hard rules
Only the two files above. Never edit `tests/golden/`, `jules/psoma.md` or
`examples/`. No new deps. `cargo test` must pass.
