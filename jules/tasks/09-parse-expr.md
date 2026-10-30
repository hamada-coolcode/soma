# Task 09 — parser: expressions + statements

Read `jules/grammar.ebnf` (§expressions, §statements) and
`src/ast/expr.rs`, `src/parse/{util.rs,expr.rs}`.

## Files you may touch (ONLY these)
- `src/parse/expr.rs`

## Goal
Implement `expr(p, min_prec) -> Result<Expr, ParseError>` and `stmt(p)`.

Precedence (low→high): lambda `x => e` / `(a b) => e`; `or`; `and`; unary `not`;
range `..b` / `a..b`; `==` `!=`; `<` `>` `<=` `>=`; `+` `-`; `*` `/` `%`; unary
`-`; postfix (call `(args)`, index `[e]`, field `.name`, propagate `!`);
primary.

Primary: int/float/string/fstring/multiline, `true`/`false`/`null`, `self`,
name, `(expr)`, list/comprehension:
`[a b c]` → `ExprKind::List`; `[e for x in it]` → `ExprKind::Comprehension`;
`if cond => a else b` → `ExprKind::If` (one-liner form; blocks also allowed via
Indent/Dedent).
Calls: args space-separated, optional `name = expr` → `Arg { name: Some(..) }`.
Statements: expression stmt, `x = e` / `x.f = e` / `x[i] = e` → `Assign`,
`while`, `for`, `loop`, `break`, `continue`, `return [e]`, `match` (arms:
`Pattern => stmt`), and `if` in statement position.
`x!` → `ExprKind::Propagate`. A `!` followed by more tokens on the same line is
a handler — represent it as `Propagate` and leave handler lowering to desugar
(note it in a comment).

## Tests
Fixtures asserting full AST equality (`assert_eq!`) for: arithmetic precedence
(`1 + 2 * 3`), unary minus, `not a and b`, range `..4` and `1..4`, method chain
`a.b().c`, index, call with named arg, list literal, comprehension, lambda
single/multi, `return x!`, assignment to field and index, `while`/`for`/`loop`
with Indent/Dedent fixture tokens, `match` with two arms.

## Hard rules
Only edit `src/parse/expr.rs`. Never edit `Cargo.toml`, any `mod.rs`,
`design/`, `examples/`, `jules/`. Keep both public signatures.
