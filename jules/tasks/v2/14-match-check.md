# Task 14 — Match typing + exhaustiveness

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/types/check_match.rs`

## Goal
Check `match`:

- scrutinee must be an enum/error-union/optional/bool;
- each arm pattern is a valid variant of the scrutinee's type, binds match the
  payload arity and get the payload types;
- `_` is allowed once and only as the last arm (otherwise warn);
- exhaustiveness: every variant covered, or a `_` arm present → else `E0501`
  listing the missing variants;
- unreachable arm after `_` → warning diagnostic;
- arms must produce the same type when the match is used as an expression.

## Acceptance (definition of done)
- Unit tests: exhaustive ok, missing variant → `E0501` with the variant list,
  duplicate arm, wrong bind arity, unreachable arm warning.
- Works on hand-built ASTs, no parser dependency.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`, `design/`,
  `examples/`, `jules/`, or `tests/golden/` (frozen).
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass.
- Do not leave `todo!()`, `unimplemented!()` or `panic!()` reachable from
  library code.
