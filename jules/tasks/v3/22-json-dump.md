# Task 22 — AST JSON dump

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/emit_json.rs`
- `tests/json_dump.rs`

## Goal
`emit_json(&[Item]) -> String`: stable, pretty-printed (2-space) JSON of the
AST, hand-rolled (no serde), deterministic key order, spans included as
`{"start":n,"end":n}`.

- cover every `ItemKind`, `ExprKind`, `StmtKind`, `TypeKind`, `PatternKind`
  variant that exists today; unknown/`Null` kinds emit `{"kind":"unknown"}`;
- escape `"`, `\`, `\n`, `\t`, and control chars < 0x20 correctly;
- 4-space→2-space nested indentation, one key per line, trailing newline.

## Acceptance (definition of done)
- `tests/json_dump.rs`: a snapshot string for `tests/golden/arith.soma`;
  a tiny in-test JSON validator (bracket/quote/escape balance) run over the
  dump of all 14 examples; escaping tests for tricky string literals.
- `cargo run -- parse FILE` is untouched (CLI wiring by the integrator).

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`,
  `design/`, `examples/`, `jules/`, or `tests/golden/`.
- Keep the existing `pub fn emit_json` signature.
- No new dependencies. `cargo test` must pass. No `todo!()`/`unimplemented!()`.
