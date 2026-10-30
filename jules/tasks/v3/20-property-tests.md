# Task 20 — Lexer/parser property tests

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `tests/props.rs`
- `tests/corpus/fuzz/**` (new files only)

## Goal
Prove the front end never panics, never hangs, and is deterministic.

- deterministic xorshift64* generator (seed printed in every failure message);
- 500 random byte strings (valid UTF-8, length 0..2000) and 300 structured
  mutants of real examples (delete/duplicate/replace a token, unbalance a
  delimiter, truncate mid-string);
- for each input: `lexer::scan` then `parse::parse_file` must return `Ok` or
  `Err` — no panic, no timeout; running twice must give identical results;
- a corpus of hand-written nasty files (`tests/corpus/fuzz/*.soma`): unclosed
  brace, `#` at EOF, 1000-level indent, CRLF input, lone `[`, `match` with no
  arms, 1 MB identifier.

## Acceptance (definition of done)
- Suite finishes in under 5 seconds in debug mode.
- No test depends on network, clock or filesystem order (sort directory reads).
- Every corpus file is also asserted to produce a diagnostic, not a panic.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`,
  `design/`, `examples/`, `jules/`, or `tests/golden/`.
- No new dependencies (no `proptest`, no `rand`).
- `cargo test` must pass.
