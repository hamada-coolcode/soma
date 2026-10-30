# Task 03 — Lexer hardening

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/lexer/scan.rs`
- `src/lexer/literal.rs`
- `src/lexer/indent.rs`

## Goal
Make the lexer robust on real files:

- CRLF line endings (treat `\r\n` as one newline; no stray `\r` tokens);
- UTF-8 BOM at byte 0 is skipped;
- file without trailing newline still ends with `Eof` (and correct Dedents);
- empty file → just `Eof`; file of only blank lines/comments → just `Eof`;
- unterminated string / f-string / multiline → `LexError` with a useful span,
  never a panic or a silent success;
- `#` comment at EOF without newline is fine; `#[` is always `AttrOpen`;
- indentation errors report the offending line's width and the open levels;
- deeply nested indentation (100 levels) works.

Do not change the public functions

## Acceptance (definition of done)
signatures or the `Token` shape.
- 25+ unit tests, including a table of bad inputs asserting `is_err()`.
- Property-ish test: for 200 generated token sequences, `scan` never panics.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`, `design/`,
  `examples/`, `jules/`, or `tests/golden/` (frozen).
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass.
- Do not leave `todo!()`, `unimplemented!()` or `panic!()` reachable from
  library code.
