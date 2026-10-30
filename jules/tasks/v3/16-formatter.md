# Task 16 — `soma fmt` canonical printer

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/fmt/mod.rs`
- `src/fmt/print.rs`
- `tests/fmt.rs`

## Goal
Print the AST back as canonical soma source (`format_items(&[Item]) -> String`).

Style rules (from `grammar.ebnf` + `design/`):
- 2 spaces per indent level; one blank line between top-level items;
- `fn` sections: one `fn` header, then member declarations at +1 indent, bodies
  at +2;
- no parentheses or commas around parameter lists in **declarations**; calls
  keep `(a b c)` with spaces, no commas;
- `->` before return types; generics in `[]`; member access with `.`;
- list literals `[1 2 3]`, ranges `..b`; `+` preserved verbatim (never rewrite
  to `.add()`);
- match arms one per line; `if`/`elif`/`else` one-liners stay one line when the
  body is a single expression;
- attributes keep their own line(s), above the member they annotate.

## Acceptance (definition of done)
- `tests/fmt.rs`: for every `examples/*/main.soma`, `parse → fmt → parse`
  yields an AST that formats to the identical string (idempotence:
  `fmt(fmt(x)) == fmt(x)`).
- Snapshot test for `tests/golden/arith.soma`.
- Direct unit tests for: nested blocks, generics, lambdas, match, attributes,
  string literals with quotes/newlines escaped correctly.

## Hard rules
- Only the files listed above. Do **not** edit `src/cli.rs` — CLI wiring is
  done by the integrator.
- Never edit `Cargo.toml`, any other `mod.rs`, `design/`, `examples/`, `jules/`,
  or `tests/golden/`.
- Keep every existing `pub fn` signature. No new dependencies.
- `cargo test` must pass. No `todo!()`/`unimplemented!()`.
