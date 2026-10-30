# Task 17 — Module loader + directory mode

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/loader.rs`
- `src/driver.rs`
- `src/cli.rs`
- `tests/loader.rs`

## Goal
Resolve `import lib:std → io → term → output` style imports to files:

- `loader::load_entry(path)` loads the entry file plus every transitively
  imported module; paths come from `lib/std/<segments>.soma` (or the
  workspace-relative equivalent);
- each loaded file appears once (dedupe by canonical path); record its
  display path plus source text;
- missing module → `E0701` including the searched paths in `with_help`;
- import cycle → `E0702` naming the cycle;
- `soma check <dir>` walks `**/*.soma` (skip `target/`, hidden dirs), checks
  each file independently, prints a summary line
  (`N files, E errors, W warnings`) and exits 1 if any file failed;
- `soma parse`/`soma check` accept multiple file arguments.

## Acceptance (definition of done)
- `tests/loader.rs`: fixture tree in `tests/loader_fixtures/` — a→b→c chain,
  duplicate import (loaded once), missing module, cycle, and a directory run
  where one file has an error and two are clean (summary + exit code).
- Existing single-file behaviour and exit codes 0/1/2 unchanged.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any other `mod.rs`,
  `design/`, `examples/`, `jules/`, or `tests/golden/`.
- `src/cli.rs` is yours in this wave — no other task touches it.
- Keep every existing `pub fn` signature. No new dependencies.
- `cargo test` must pass. No `todo!()`/`unimplemented!()`.
