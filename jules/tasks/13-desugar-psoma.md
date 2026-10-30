# Task 13 — desugar: `.psoma` emitter

Read `design/08-pipeline.md` (§preprocessor) and `src/desugar/psoma.rs`,
`src/ast/**`.

## Files you may touch (ONLY these)
- `src/desugar/psoma.rs`

## Goal
Implement `emit(items: &[Item]) -> String`: print the lowered AST as `.psoma`
— an explicit, regular form used only for debugging (`--emit-psoma`):
- braces `{ }` for blocks, `;` after statements, `,` between params/args/items
- no indentation significance (indent for readability with 2 spaces)
- signatures: `name(generics)[p: T, q: U] -> Ret`? No — keep the design's
  surface: `name p: T, q: U -> Ret` but with commas; params in parens:
  `name(p: T, q: U) -> Ret`
- expressions fully parenthesized where precedence is ambiguous
- `Error!Type` printed as-is; `Propagate(e)` as `propagate(e)`
- deterministic: same AST → same string, always. End with exactly one newline.

Also add `pub fn emit_item(item: &Item) -> String` for golden tests.

## Tests
Golden strings for: `fn add a: int b: int -> int { return a + b; }`,
an `object` with one field + one method, an `if/elif/else`, a `match` with two
arms, a call with args, a list literal, a lambda. Exact `assert_eq!` on strings.

## Hard rules
Only edit `src/desugar/psoma.rs`. Never edit `Cargo.toml`, any `mod.rs`,
`design/`, `examples/`, `jules/`. Keep both public signatures.
