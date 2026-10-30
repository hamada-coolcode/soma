# Task 01 — CLI driver: parse / check / run

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/cli.rs`
- `src/driver.rs`
- `tests/cli.rs` (create)

## Goal
Turn the binary into a real compiler front-end command:

```
soma parse <file> [--emit-psoma]   # lex+parse; --emit-psoma prints lowered IR
soma check <file>                  # parse + resolve + type check, all diagnostics
soma run <file>                    # delegates to interp::run_file (stub today)
soma --version | --help
```

Exit codes: `0` success, `1` diagnostics, `2` usage/IO error.
`driver::parse_source`, `driver::check_source`, `driver::emit_psoma`,
`driver::render` are used by other tasks: keep their signatures.

`check` should render **every** diagnostic it has (one per block) and return 1.
Support `--` separator before program args (stored, not yet used).

## Acceptance (definition of done)
- `tests/cli.rs` drives the real binary via `env!("CARGO_BIN_EXE_soma")`:
  `--version`, `parse` ok, `parse --emit-psoma` non-empty, `parse` on a syntax
  error → exit 1 + `error`, missing file → exit 2, `check` on an example → ok.
- `soma parse examples/hello-world/main.soma` exits 0.
- `driver` has no panics; IO errors become diagnostics.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`, `design/`,
  `examples/`, `jules/`, or `tests/golden/` (frozen).
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass.
- Do not leave `todo!()`, `unimplemented!()` or `panic!()` reachable from
  library code.
