# Task 23 — "did you mean" + code help

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/diag/suggest.rs`
- `tests/suggest.rs`

## Goal
Pure helper functions (no I/O):

- `did_you_mean(name, candidates) -> Option<String>`: Levenshtein distance
  ≤ 2 for names longer than 3 chars (≤ 1 for shorter), case-sensitive pass
  first, then a case-insensitive retry; ties broken lexicographically; returns
  `None` when nothing is close;
- `suggest_help(code) -> Option<&'static str>`: one-line help for every code the
  compiler emits today — `E0301`–`E0304`, `E0500`–`E0505`, `E0701`–`E0702`,
  `W0101`–`W0103`, `E0400`.

## Acceptance (definition of done)
- `tests/suggest.rs`: table-driven — `undefind_name`→`undefined_name`,
  `reslts`→`results`, no suggestion beyond distance 2, short-name behaviour,
  tie-break, case-insensitive fallback; `suggest_help` covers every code above
  and returns `None` for garbage.
- Empty candidate slice and empty name must not panic.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`,
  `design/`, `examples/`, `jules/`, or `tests/golden/`.
- Wiring into the resolver is done by the integrator.
- Keep the existing `pub fn` signatures. No new dependencies.
- `cargo test` must pass.
