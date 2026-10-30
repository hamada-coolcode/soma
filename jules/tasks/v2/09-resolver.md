# Task 09 — Resolver: scopes, mutability, name errors

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/resolve/mod.rs`
- `src/resolve/scope.rs`
- `src/resolve/symbol.rs`

## Goal
Implement `resolve::check(items) -> Vec<Diagnostic>`:

- build scopes: file → items → fn/object bodies → blocks;
- `Param` symbols are **immutable**: assigning to a parameter is `E0301`
  ("parameters are read-only; bind a new local");
- locals are mutable; `self` is mutable; fields resolve on `self` only
  (bare `x` never resolves to a field);
- undefined names → `E0302`; duplicate definitions in the same scope →
  `E0303`; shadowing an outer scope is allowed;
- functions/objects/types are visible file-wide (order-independent);
- imports register module paths;
- one diagnostic per occurrence, with the expression's span.

## Acceptance (definition of done)
- Unit tests with hand-built ASTs for every bullet.
- `examples/minigrep/main.soma` and `hello-world` resolve with zero diagnostics.
- A fixture assigning to a parameter yields exactly one `E0301` with the span of
  the assignment target.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`, `design/`,
  `examples/`, `jules/`, or `tests/golden/` (frozen).
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass.
- Do not leave `todo!()`, `unimplemented!()` or `panic!()` reachable from
  library code.
