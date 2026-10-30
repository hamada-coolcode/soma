# Task 11 — Type checker: expressions

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/types/check_expr.rs`

## Goal
Implement `check(table, expr) -> Result<TypeId, Diagnostic>` using
`types::unify` and the resolver's symbols:

literals, names, `self`, lists, index, field, calls (incl. object methods and
builtin paths), unary, binary (with `+` on lists), range, lambda, if-expression,
comprehension, propagate.

Diagnostics use `DiagCode` (`E0400` family): mismatch, not callable, wrong
arity, unknown field/method name with a "did you mean" list. Every diagnostic
carries the expression span.

## Acceptance (definition of done)
- Unit tests over hand-built ASTs for each form above, including two failures
  (`1 + "x"`, calling a non-function).
- No `unwrap()` on type lookups; missing types produce diagnostics.
- Works when `resolve` has not been run: return a diagnostic, never panic.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`, `design/`,
  `examples/`, `jules/`, or `tests/golden/` (frozen).
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass.
- Do not leave `todo!()`, `unimplemented!()` or `panic!()` reachable from
  library code.
