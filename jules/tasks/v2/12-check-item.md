# Task 12 — Type checker: items and functions

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/types/check_item.rs`

## Goal
Drive the checker over a file:

- function signatures: duplicate param names, parameter types must exist,
  generic params must be declared;
- body checking via `check_expr`, with locals/params bound to their declared
  types; assignments require the same type (params already rejected by resolve);
- `return` type checking; missing `return` on a non-`!`/non-unit function is an
  error; `main -> !` accepts `return` with no value;
- `!` propagation requires a fallible expression (`E!T`); `e!` in a function
  whose return type cannot carry the error is `E0400`;
- object methods get `self` typed as the object; field initialisers check
  against declared field types;
- const values are checked; unify against nothing else.

## Acceptance (definition of done)
- `types::check` returns diagnostics, not panics, for every malformed file in
  `tests/corpus/`.
- Tests: correct/incorrect return, param type mismatch, `main -> !` with `!`,
  propagation outside a fallible fn.
- `types::check(&[])` is empty and `check` is deterministic.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`, `design/`,
  `examples/`, `jules/`, or `tests/golden/` (frozen).
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass.
- Do not leave `todo!()`, `unimplemented!()` or `panic!()` reachable from
  library code.
