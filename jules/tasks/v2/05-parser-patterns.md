# Task 05 — Parser: match patterns

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/parse/pattern.rs`

## Goal
Extend `pattern.rs` (used by `stmt`'s `match` handling via `match_arm`):

- wildcard `_` (encoded as `Pattern { name: "_", binds: [] }`);
- dotted variants: `Expr.Num(v)` (encode the path in `Pattern.name` as
  `"Expr.Num"`; document it in the module docs);
- multiple binds `Pair(a b)`, zero binds `None`, nested calls rejected with a
  clear error;
- a pattern followed by `=>` is required; anything else is an error pointing at
  the token after the pattern.

Do not change the `Pattern` struct in `src/ast` — this task is parsing only.

## Acceptance (definition of done)
- `match_arm`/`pattern` unit tests for all forms above.
- One integration test parsing a `match` with 4 arms through
  `soma::parse::parse_file`.
- Malformed inputs produce `Err` with the right span (no panics).

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`, `design/`,
  `examples/`, `jules/`, or `tests/golden/` (frozen).
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass.
- Do not leave `todo!()`, `unimplemented!()` or `panic!()` reachable from
  library code.
