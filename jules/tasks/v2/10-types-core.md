# Task 10 — Types: representation + unification

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/types/ty.rs`
- `src/types/unify.rs`

## Goal
Implement the type engine:

- `Type` display (`int`, `string`, `T[]`, `T[n]`, `T?`, `(int) int`, `E!T`, `_`),
  parsing a `TypeRepr` into a `Type` (generics become `Var`s), and
  `TypeTable::resolve` following var bindings to a fixed point;
- `unify`: int/float promotion is **explicit** (no implicit coercion),
  structural matching for lists/arrays/optionals/fns/error-unions/named,
  occurs-check for `Var`, and `UnifyError` messages that name both sides;
- `Type::Named` equality is by name + args;
- idempotent interning is not required, but `resolve` must terminate.

## Acceptance (definition of done)
- 20+ unit tests: var binding, occurs check, list/optional/fn/unions,
  name mismatch, nested generics, `resolve` idempotence.
- `unify` never panics; every failure returns `UnifyError` with both types
  rendered.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`, `design/`,
  `examples/`, `jules/`, or `tests/golden/` (frozen).
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass.
- Do not leave `todo!()`, `unimplemented!()` or `panic!()` reachable from
  library code.
