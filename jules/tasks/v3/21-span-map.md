# Task 21 — psoma ↔ soma span map

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/desugar/psoma.rs`
- `src/diag/sourcemap.rs`
- `tests/span_map.rs`

## Goal
Diagnostics produced against lowered `.psoma` must map back to the `.soma`
source. Keep the emitted text byte-identical to today's goldens.

- add `psoma::emit_with_map(&[Item]) -> (String, Vec<(usize, Span)>)` where each
  entry maps a byte offset in the emitted text to the originating soma `Span`;
- `emit` stays a thin wrapper and its output must not change;
- `SourceMap::map_psoma_span(&self, offset: usize) -> Option<Span>` picks the
  nearest mapping at or before `offset`;
- offsets are strictly increasing; every emitted token start has an entry.

## Acceptance (definition of done)
- `tests/span_map.rs`: mapping an offset inside a lowered `+` append lands
  inside the original expression; mapping before the first entry returns
  `None`; all three goldens still match byte-for-byte.
- `tests/psoma_golden.rs` and `tests/examples_parse.rs` stay green untouched.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`,
  `design/`, `examples/`, `jules/`, or `tests/golden/`.
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass. No `todo!()`/`unimplemented!()`.
