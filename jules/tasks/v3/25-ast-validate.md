# Task 25 — AST structural invariants

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/ast/validate.rs`
- `tests/validate.rs`

## Goal
`validate(&[Item], src_len: usize) -> Vec<Diagnostic>` — a cheap post-parse
invariant check used by tests and (later) debug builds:

- every span satisfies `start <= end <= src_len`;
- item/field/param/method names are non-empty after trimming;
- no duplicate parameter names in a `fn`, no duplicate field or method names in
  an `object`, no duplicate `trait` method names;
- a `fn` member body is never empty (`main` may be, but other members may not);
- generic parameter lists contain no duplicates;
- nested item spans are contained in their parent's span.

Messages name the offending symbol and the rule broken.

## Acceptance (definition of done)
- `tests/validate.rs`: every `examples/*/main.soma` validates with zero
  diagnostics; one hand-built fixture per violation, each producing exactly one
  diagnostic with the right code (`E0801` span, `E0802` name, `E0803`
  duplicate, `E0804` empty body, `E0805` nesting).
- Runs over the whole example corpus in one test.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`,
  `design/`, `examples/`, `jules/`, or `tests/golden/`.
- Keep the existing `pub fn validate` signature. No new dependencies.
- `cargo test` must pass. No `todo!()`/`unimplemented!()`.
