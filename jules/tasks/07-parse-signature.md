# Task 07 — parser: signatures

Read `jules/grammar.ebnf` (§fn signatures) and `src/parse/{util.rs,signature.rs}`.

## Files you may touch (ONLY these)
- `src/parse/signature.rs`

## Goal
Implement `signature(p) -> Result<Signature, ParseError>`:
```
name [G] p: Type p: Type -> Ret
```
- bare params, **no parens, no commas**; stop at `->`, newline, or end of input
- generics in `[]` right after the name: `Stack[T]`, `input[uint]`
- `->` then `types::type_repr`
- no `->` → `ret: None`
- `Self` is a valid return type name (a `Name`, not a keyword)
Delegate types to `types::type_repr` (implemented in parallel; it returns an
error at HEAD — so type-free tests only).

## Tests
Fixtures: `main` (name only), `print` (name only), `add` with no types → error?
No: `add a: int` needs types. So test: `main`, `print`, `hello -> string` (error
until types land — skip), generics only: `foo [T]` → name + generics + no
params. Also: a param name without `:` → `Err`; `[T U]` parses two generics.

## Hard rules
Only edit `src/parse/signature.rs`. Never edit `Cargo.toml`, any `mod.rs`,
`design/`, `examples/`, `jules/`. Keep `Signature` fields and `signature` shape.
