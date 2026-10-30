# Task 08 — parser: type expressions

Read `jules/grammar.ebnf` and `design/02-syntax.md` (§Types),
`src/ast/ty.rs`, `src/parse/{util.rs,types.rs}`.

## Files you may touch (ONLY these)
- `src/parse/types.rs`

## Goal
Implement `type_repr(p) -> Result<TypeRepr, ParseError>` covering:
- `int`, `float`, `bool`, `string`, `Shape`, `Self` — `TypeKind::Named`
- `Stack[T]`, `Stack[int]` — `Named { name, generics }`
- `T[]` — `Array { elem, size: None }`
- `int[10]` — `Array { elem, size: Some(Expr::Int) }` (build the size expr
  directly from the `Int` token; do not call `expr::expr`)
- `T?` — `Optional`
- `(int) int` — `Fn { params, ret }` (params are types, space-separated inside
  the parens; empty parens = no params)
- `MathError!int` — `ErrorUnion { err, ok }` (left-assoc is not needed; only one `!`)
- bare `!` — `Never`
Postfix `[]`, `?` bind tighter than `!`. Set spans to the consumed range.

## Tests
Fixtures for every bullet above, including nested `Stack[int[]]`,
`(int int) bool`, `T?[]` (optional array of… decide and document), invalid
`[]` at start → `Err`, and `!` alone.

## Hard rules
Only edit `src/parse/types.rs`. Never edit `Cargo.toml`, any `mod.rs`,
`design/`, `examples/`, `jules/`. Keep the `type_repr` signature.
