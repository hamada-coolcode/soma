# Task 07 — .psoma emitter + golden harness

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/desugar/psoma.rs`
- `tests/psoma_golden.rs` (create)

## Goal
Implement `emit(items) -> String` exactly per `jules/psoma.md`, and add a
harness that compares `emit(parse(file))` byte-for-byte with the frozen goldens:

- `tests/golden/hello.{soma,psoma}`
- `tests/golden/arith.{soma,psoma}`
- `tests/golden/object.{soma,psoma}`

The goldens are **frozen** — if output differs, fix the emitter, not the golden.

Also required: `emit` is deterministic (two calls equal), and for all 14
`examples/*/main.soma` the output is non-empty and contains at least one `fn `.

Cover every expression/statement in the contract table (binary always
parenthesized, `if (c) { } else if (c2) { }`, `match (e) { }`, named args,
`self`, lists, lambdas, ranges, `propagate(...)`).

## Acceptance (definition of done)
- All 3 goldens match exactly, including trailing newline.
- Emitter has no `unwrap()` on user data and never panics on a valid AST.
- Unit tests per expression form, plus the golden harness.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`, `design/`,
  `examples/`, `jules/`, or `tests/golden/` (frozen).
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass.
- Do not leave `todo!()`, `unimplemented!()` or `panic!()` reachable from
  library code.
