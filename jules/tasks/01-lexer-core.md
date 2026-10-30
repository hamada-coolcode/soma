# Task 01 — lexer core: scan + indentation

Read `jules/grammar.ebnf` (§lexical, §indentation) and `src/lexer/token.rs` first.

## Files you may touch (ONLY these)
- `src/lexer/scan.rs`
- `src/lexer/indent.rs`

## Goal
Implement `scan_tokens(src) -> Result<Vec<Token>, LexError>` and `IndentTracker`.

`scan_tokens` walks bytes and emits: identifiers (`[A-Za-z_][A-Za-z0-9_]*`),
numbers via `literal::scan_number`, strings via `literal::scan_string` /
`literal::scan_fstring`, operators via `symbol::scan_op`, `#[` as
`Op(Op::AttrOpen)`, `#` to end-of-line as a comment, spaces skipped, each
physical newline as `TokKind::Newline`. Every token carries its byte `Span`.

`IndentTracker` implements the algorithm in grammar §indentation: stack starting
at `[0]`; deeper line → one `Indent`; shallower → one `Dedent` per level popped;
width matching no open level → `LexError`; tabs in indentation → `LexError`;
`finish()` pops the rest.

At EOF: remaining `Dedent`s, then `Eof`.

## Tests (must pass with `cargo test`)
Tests live in the same file. NOTE: `literal.rs` and `symbol.rs` are implemented
in parallel and return `None` at HEAD — so your `scan_tokens` tests must use
inputs made only of identifiers, spaces, newlines and indentation, e.g.
`"a b\n  c\n"`. Test `IndentTracker` directly for every branch (deeper, equal,
shallower, bad width, tabs).

## Hard rules
Only edit the two files above. Never edit `Cargo.toml`, any `mod.rs`, `design/`,
`examples/`, `jules/`. No new deps.
