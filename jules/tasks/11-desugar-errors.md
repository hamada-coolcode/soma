# Task 11 — desugar: `!` propagate and handler

Read `design/05-errors.md` and `src/desugar/errors.rs`.

## Files you may touch (ONLY these)
- `src/desugar/errors.rs`

## Goal
Lower error sugar on `ExprKind::Propagate(inner)`:
- Plain `e!` (no handler): rewrite to an early-return match: if `e` is an error
  variant, `return <error>`; else use the success value. Since the analyzer
  doesn't exist yet, emit a *structural* form:
  `If { cond: Call(is_error, [e]), then: block[ Return(e) ], else_: None }`
  and replace the expression with the success value — document the chosen
  encoding in a module comment. Keep it deterministic and testable.
- Handler `e ! handler`: the parser represents the handler as... (check
  `src/parse/expr.rs` docs; at HEAD it does not exist yet). Support both:
  (a) `Propagate(inner)` whose `inner` is a `Call` and the *next sibling
  statement* is the handler, and (b) `Propagate(inner)` alone = plain propagate.
  For (a), rewrite to `If { cond: is_error(e), then: block[ handler ], else_:
  Some(block with e's value) }`.
- Recurse everywhere (items, blocks, statements, expressions).
- Never touch `main`'s signature; only bodies.

## Tests
Hand-built ASTs: plain propagate; handler sibling; nested inside `for`;
`Call(is_error, ..)` naming is per your module comment; ensure no propagate
nodes remain after `rewrite`.

## Hard rules
Only edit `src/desugar/errors.rs`. Never edit `Cargo.toml`, any `mod.rs`,
`design/`, `examples/`, `jules/`. Keep `rewrite(items) -> Vec<Item>`.
