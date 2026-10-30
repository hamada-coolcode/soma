# Task 12 — desugar: comprehensions and `for … in`

Read `design/02-syntax.md` (§List comprehensions) and
`src/desugar/comprehension.rs`.

## Files you may touch (ONLY these)
- `src/desugar/comprehension.rs`

## Goal
- `ExprKind::Comprehension { expr, var, iter }` →
  `Block`-free structural form: `Call { callee: Call(__comp, [iter, Lambda(x => e)]) }`
  is NOT allowed (no such builtin). Instead emit a `List`-building helper call
  `Call { callee: Name("__comprehension"), args: [Lambda { params: [var], body: expr }, iter] }`
  and document the encoding. Deterministic, no spans lost.
- `StmtKind::For { var, iter, body }` → `While` with an iterator cursor:
  `{ __it = iter; while __it.has_next() { var = __it.next(); body } }` using
  `Assign` + `While` nodes. Document the cursor convention.
- Recurse everywhere; leave `while`/`loop` alone.

## Tests
Hand-built ASTs: comprehension → helper call shape; `for` → while with cursor
assignments; nested comprehension inside a function body; span preservation for
the comprehension expression.

## Hard rules
Only edit `src/desugar/comprehension.rs`. Never edit `Cargo.toml`, any `mod.rs`,
`design/`, `examples/`, `jules/`. Keep `rewrite(items) -> Vec<Item>`.
