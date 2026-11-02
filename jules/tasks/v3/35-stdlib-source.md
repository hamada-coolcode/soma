# Task 35 — Stdlib in soma: collections + optional

Read `jules/README.md`, `design/07-collections.md` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `lib/std/collections.soma` (new file)
- `lib/std/optional.soma` (new file)
- `tests/stdlib_source.rs` (new file)

## Goal
Write real soma source for two stdlib modules, using only syntax the parser and
formatter already accept (no new features):
- `collections.soma`: `Pair[A B]` with `first`/`second`, `Stack[T]` with
  `push`/`pop`/`peek`/`size`, `Queue[T]` with `push`/`pop`/`size`;
- `optional.soma`: `Optional[T]` object with `Some`/`None` variants, `map`,
  `unwrap_or`, `is_some`;
- both files include `test "..."` blocks exercising the logic.

## Acceptance
- `tests/stdlib_source.rs`: each file parses with zero diagnostics, emits
  non-empty `.psoma`, passes `ast::validate`, and the formatted form re-parses to
  the same string.
- No parser changes, no new syntax.

## Hard rules
Only the three files above. No new deps. `cargo test` must pass.
