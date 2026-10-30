# Task 06 — Diagnostics: stable error codes

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/diag/code.rs` (create)
- `src/diag/error.rs`
- `src/diag/render.rs`
- `tests/diagnostics.rs` (create)

## Goal
Give every diagnostic a stable machine-readable code:

```
E0100 lex      E0200 parse
E0300 resolve  E0301 read-only parameter
E0302 undefined name  E0303 duplicate definition
E0400 type     E0401 mismatch  E0402 not callable  E0403 wrong arity
E0500 match    E0501 non-exhaustive
```

- `pub enum DiagCode` with `as_str()` (`"E0301"`) and `DiagCode::parse`.
- `Diagnostic` gains `pub code: Option<DiagCode>`; constructors keep working
  (`Diagnostic::new` sets it, `with_code` overrides).
- `render` prints `error[E0301]: message` when a code is set.
- `tests/diagnostics.rs` asserts rendering for one code per family, and that
  codes round-trip through `as_str`/`parse`.

## Acceptance (definition of done)
- `src/driver.rs` unchanged API; it may set `E0100`/`E0200` by calling
  `Diagnostic::new(...).with_code(...)` from **inside `diag`** — instead, add
  `Diagnostic::lex/parse` helpers so other modules stay untouched.
- No numeric codes are invented at call sites: only `DiagCode` values.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`, `design/`,
  `examples/`, `jules/`, or `tests/golden/` (frozen).
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass.
- Do not leave `todo!()`, `unimplemented!()` or `panic!()` reachable from
  library code.
