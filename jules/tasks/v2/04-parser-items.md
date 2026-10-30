# Task 04 — Parser: item bodies (object/trait/enum/import/const/test)

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/parse/items.rs`
- `src/parse/blocks.rs`

## Goal
Move the `object`, `trait`, `enum`, `import`, `const` and `test` parsing out
of `blocks.rs` into `items.rs`, one `pub(crate) fn` per item kind, and complete
them:

- `object`: multiple declarations per section, fields vs methods, generics,
  attributes on members;
- `trait`: method sets with signatures only (no bodies);
- `enum`: variants with 0..n payload types;
- `import`: nested path trees (`lib:std` → `io` → `term` → `output`);
- `const`: multiple `NAME = expr` per block;
- `test`: `"name"` + block, several per section.

`blocks.rs` keeps dispatch + the `fn` section only. Keep `parse_file`

## Acceptance (definition of done)
exactly as it is.
- Unit tests for each item kind, in `items.rs`, using `Token::ident/op/...`.
- Round-trip: every `examples/*/main.soma` parses and items land in the
  right `ItemKind` (assert counts per example).
- Errors point at the first token of the malformed item.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`, `design/`,
  `examples/`, `jules/`, or `tests/golden/` (frozen).
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass.
- Do not leave `todo!()`, `unimplemented!()` or `panic!()` reachable from
  library code.
