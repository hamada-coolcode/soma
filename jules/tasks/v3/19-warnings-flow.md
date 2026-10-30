# Task 19 — Warnings + flow analysis

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/resolve/warnings.rs`
- `src/resolve/flow.rs`
- `src/resolve/mod.rs`
- `tests/warnings.rs`

## Goal
Two passes over the parsed items, returning `Vec<Diagnostic>` (warnings use
`ErrorKind::Other` and codes `W…`):

`warnings::check`:
- unused local (`W0101`) — bound but never read after binding;
- unused import (`W0102`) — imported module with no member reference;
- `_`-prefixed names are intentionally-unused and never warned.

`flow::check`:
- use-before-assign (`E0304`) — reading a local on a path where it may not be
  assigned yet;
- unreachable code after `return`/`break`/`continue` (`W0102`-style `W0103`),
  reported once per dead region, not per statement.

Be conservative: a variable assigned in every branch of `if`/`elif`/`else`
counts as assigned; loops may execute zero times, so assignments inside a loop
body do not leak out.

## Acceptance (definition of done)
- `tests/warnings.rs`: one fixture per rule, plus no-false-positive cases
  (both-branch assignment, `_unused`, loop-carried assignment).
- `examples/minigrep` yields zero warnings and zero flow errors.
- Wire both passes into `resolve::check` so `soma check` prints them.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any other `mod.rs`,
  `design/`, `examples/`, `jules/`, or `tests/golden/`.
- Keep every existing `pub fn` signature; additive only.
- No new dependencies. `cargo test` must pass. No `todo!()`/`unimplemented!()`.
