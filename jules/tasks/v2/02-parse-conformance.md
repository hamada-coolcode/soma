# Task 02 — Parser conformance corpus

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `tests/conformance_parse.rs` (create)
- `tests/corpus/*.soma` (create, 10 small files)

## Goal
Add a corpus of small programs that exercise the **whole documented surface**
(one feature per file) and a harness that locks parser invariants:

1. every `tests/corpus/*.soma` and every `examples/*/main.soma` lexes + parses;
2. parsing is deterministic (same input twice → equal ASTs);
3. every item has a non-empty span, and spans are ordered (`start <= end`);
4. `fn`/`object` items carry bodies; `test "..."` blocks exist where declared;
5. identifiers inside a file are not duplicated at the same declaration level.

Corpus files to include: hello, arithmetic, lists+comprehension, ranges,
optionals, error-union + `!` + handler, objects+self, traits+enums+match,
generics (`Stack[T]`, `T[]`, `T[n]`, `soma.new[int]()`), attributes+async.

## Acceptance (definition of done)
- `tests/corpus/` has exactly the 10 files above, each parsing successfully.
- Failure messages name the file, the offending byte offset and the expectation.
- Harness uses `soma::driver::parse_source` only — no CLI, no shelling out.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`, `design/`,
  `examples/`, `jules/`, or `tests/golden/` (frozen).
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass.
- Do not leave `todo!()`, `unimplemented!()` or `panic!()` reachable from
  library code.
