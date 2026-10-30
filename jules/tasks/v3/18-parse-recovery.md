# Task 18 — Parser error recovery

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/parse/recover.rs`
- `src/parse/expr.rs`
- `src/parse/blocks.rs`
- `tests/parse_errors.rs`

## Goal
Make the parser report several errors per file instead of aborting on the first.

- `parse_file_recovering(toks) -> (Vec<Item>, Vec<Diagnostic>)` returns the
  items it could parse plus every error; cap at 20 diagnostics per file, then
  stop with a final "too many errors" diagnostic.
- Resync at declaration boundaries (`Newline`/`Dedent` at top level, next `fn`
  member, next `object`/`trait`).
- Old `parse_file` keeps its signature and still returns the first error.
- Never loop forever: every iteration must consume a token or emit an error and
  advance.

## Acceptance (definition of done)
- `tests/parse_errors.rs`: a file with three broken functions yields exactly
  three errors, one per function, and still contains the good functions; a file
  of pure garbage terminates; recovery works inside `object` bodies.
- Recovered items are structurally valid (no half-built `Fn` with empty body).

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`,
  `design/`, `examples/`, `jules/`, or `tests/golden/`.
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass. No `todo!()`/`unimplemented!()`.
