# Task 10 — desugar: list `+`

Read `design/02-syntax.md` (§List `+`) and `src/desugar/append.rs`.

## Files you may touch (ONLY these)
- `src/desugar/append.rs`

## Goal
Rewrite `L + x` (and `L + M`) on list/string receivers into method calls:
- left operand is an `Expr` that is not numeric-literal/known-int → rewrite
  `Binary { Add, lhs, rhs }` to
  `Call { callee: Field { base: lhs, name: "add" }, args: [rhs] }`.
- Chained `a + b + c` (left-assoc) → `a.add(b).add(c)`.
- **Do not** rewrite when the LHS is a string literal (concatenation of two
  literals stays `Add` — the type checker will reject it; document why).
- Recurse into every expression/statement/block/item (write a small `walk`
  module-internal helper). Preserve spans.

Since types aren't available yet, use this rule: rewrite `lhs` if it is NOT
`ExprKind::Int`/`Float`. (`.add` vs `.extend` is chosen by the type checker
later — the call target name is always `add`; extend-list is a later pass.)

## Tests
Hand-built ASTs (no lexer/parser needed): `seq + 5` → call; `seq + [1 2]` →
call; `1 + 2` unchanged; nested `f(a + b)` rewritten inside; `a + b + c`
chains; walk reaches into `if`/`while`/`for` bodies and object methods.

## Hard rules
Only edit `src/desugar/append.rs`. Never edit `Cargo.toml`, any `mod.rs`,
`design/`, `examples/`, `jules/`. Keep `rewrite(items) -> Vec<Item>`.
