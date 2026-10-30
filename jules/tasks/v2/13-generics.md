# Task 13 — Generics: instantiation rules

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/types/generic.rs`

## Goal
Type generic declarations and instantiations:

- `Stack[T]` declaration introduces `T` as a type parameter in scope for its
  fields/methods; using an undeclared parameter is `E0400`;
- instantiation `Stack[int]` substitutes parameters for arguments; arity
  mismatch is reported (`Stack` needs 1 argument);
- `T[]`, `T[n]`, `T?` accept any element type; `T[n]` size must be a
  non-negative integer literal;
- `soma.new[int]()` / `input[uint]()` / `v.parse[int]()` use the same
  substitution path as `Stack[int]` and must not be treated as indexing;
- instantiating a non-generic type with arguments is an error.

## Acceptance (definition of done)
- Unit tests for: generic identity (`fn id[T] x: T -> T` style), nested
  instantiation `Stack[Stack[int]]`, arity errors, undeclared parameter,
  `T[n]` with a bad size, and use-site instantiation of a builtin.
- No AST changes; operate on `TypeRepr` + `TypeTable`.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`, `design/`,
  `examples/`, `jules/`, or `tests/golden/` (frozen).
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass.
- Do not leave `todo!()`, `unimplemented!()` or `panic!()` reachable from
  library code.
