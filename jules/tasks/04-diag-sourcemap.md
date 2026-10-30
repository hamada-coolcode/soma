# Task 04 — diag: source map

Read `design/08-pipeline.md` (span mapping) and `src/diag/sourcemap.rs`.

## Files you may touch (ONLY these)
- `src/diag/sourcemap.rs`

## Goal
Harden `SourceMap` (keep the existing public API: `new`, `add`, `source`,
`name`, `line_col`, `FileId`). Add:
- `pub fn file_count(&self) -> usize`
- `pub fn slice(&self, id: FileId, span: Span) -> &str` (clamped, never panics)
- `pub fn offset_at(&self, id: FileId, line: usize, col: usize) -> Option<usize>`
  (1-based line and column; returns `None` for out-of-range)
- `pub fn map_span(&self, from: FileId, to: FileId, span: Span) -> Option<Span>`
  — placeholder for `.psoma`→`.soma` mapping: map by (line, col) if `to` has
  same-or-larger coordinates, else `None`. Document that the real mapping will
  come from span annotations in the preprocessor.

`line_col` must be correct for: offset 0, offsets at `\n`, multi-line text,
offset == len, offset > len (clamp).

## Tests
Boundary tests for `line_col`, round-trip `offset_at(line_col(o)) == o`,
`slice` clamping, `map_span` same/shorter/longer inputs, `file_count`.

## Hard rules
Only edit `src/diag/sourcemap.rs`. Never edit `Cargo.toml`, any `mod.rs`,
`design/`, `examples/`, `jules/`. No new deps.
