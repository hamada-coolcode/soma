# Task 02 — lexer literals: numbers, strings, f-strings

Read `jules/grammar.ebnf` (§lexical) and `src/lexer/token.rs`.

## Files you may touch (ONLY these)
- `src/lexer/literal.rs`

## Goal
Implement the three functions already declared:
- `scan_number(src, start) -> Option<(Token, usize)>` — `int` and `float`
  (`12`, `3.5`); no exponents, no hex. `None` if the byte at `start` isn't a digit.
- `scan_string(src, start) -> Result<Option<(Token, usize)>, LexError>` —
  `"..."` with escapes `\n \t \r \\ \" \{ \}`, and `"""..."""` multiline
  (content taken verbatim, no escape processing). Unterminated → `LexError`.
- `scan_fstring(src, start) -> Result<Option<(Token, usize)>, LexError>` —
  `f"..."` producing `TokKind::FStr(Vec<FPart>)`: literal chunks as
  `FPart::Text`, each `{...}` as `FPart::Expr(raw inner source)`. The inner
  expression may itself contain balanced braces and string literals — track
  depth and quotes. `{{`/`}}` are not special (only `{}`).

Return `None` if the source at `start` doesn't begin the construct.

## Tests
Direct unit tests per function: happy paths, escapes, multiline, f-string with
nested `{ a + b }`, f-string with a string containing `}`, unterminated string
(must error with a span), non-matching first byte (`None`).

## Hard rules
Only edit `src/lexer/literal.rs`. Never edit `Cargo.toml`, any `mod.rs`,
`design/`, `examples/`, `jules/`. No new deps.
