# Task 06 — parser: block structure

Read `jules/grammar.ebnf` (§items) and `src/parse/{util.rs,mod.rs}`.

## Files you may touch (ONLY these)
- `src/parse/blocks.rs`

## Goal
Implement `parse_file(toks: &[Token]) -> Result<Vec<Item>, ParseError>`.
Handle, in this order: `#[attr]` prefixes, optional `export`, then one of
`fn | object | trait | error | enum | import | const | test`.

- `import`: nested ident path (`lib:std` = one `Name` per path segment; `:` in
  a path node joins into a single name segment), recursion for children via
  `Indent`/`Dedent`.
- `error`: name + variant names.
- `object`: name `[generics]`, then `field: Type` vs `method` decided by whether
  the member's next token is `:` (field) or a newline/signature.
- delegate to `signature::signature`, `types::type_repr`, `expr::expr`,
  `expr::stmt`. They are implemented in parallel and currently return errors —
  so your tests may only exercise `import`, `error`, and `fn`/`object`
  skeletons WITHOUT field types or bodies.
- Attributes attach to the following `Item` (`Item.attrs`).
- Spans: join first..last token spans.

## Tests
Token fixtures (see `Token::ident` etc.): an `import` tree two levels deep, an
`error` block with 2 variants, an object with only a method name that has no
params/return type (e.g. `object Foo` → `print`), and malformed input → `Err`.

## Hard rules
Only edit `src/parse/blocks.rs`. Never edit `Cargo.toml`, any `mod.rs`,
`design/`, `examples/`, `jules/`. Keep the `parse_file` signature.
