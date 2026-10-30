# Task 14 — Match typing + exhaustiveness

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/types/check_match.rs`

## Goal
Type a `match` expression and prove it total:

- the scrutinee must be an object/enum-like type, an error union, `T?`, or
  `bool`; anything else is `E0500`;
- object/enum arms bind the variant payload with the right type; wrong arity
  or wrong pattern shape is `E0502`;
- missing variants are `E0501`, and the message must name **every** missing
  variant, comma-separated, in declaration order;
- `_` is a catch-all, legal only in the last position (earlier `_` is `E0503`);
- arms after a `_` are unreachable — report `E0504` for each;
- all arm bodies must unify to one result type (`unify::unify`), mismatch is
  `E0505` with both types in the message.

## Acceptance (definition of done)
- Unit tests using hand-built `Expr`/`TypeRepr` fixtures for: exhaustive ok,
  missing one variant, missing several (message lists all), duplicate arm,
  wrong payload arity, `_` not last, unreachable arm, unifying arms.
- No AST changes; operate on `TypeRepr` + `TypeTable`.

## Hard rules
- Only the file listed above. Never edit `Cargo.toml`, any `mod.rs`, `design/`,
  `examples/`, `jules/`, or `tests/golden/`.
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass.
- No `todo!()`, `unimplemented!()` or reachable `panic!()`.
