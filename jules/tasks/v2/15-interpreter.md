# Task 15 — Interpreter: soma run

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/interp/mod.rs`
- `src/interp/value.rs`
- `src/interp/eval.rs`

## Goal
Make `soma run FILE` execute a program:

- reuse `driver::parse_source` then `desugar::lower` on the AST;
- evaluate: literals, names, lists, binary/unary, field, index, calls to
  user fns and objects (methods with `self`), `if`/`while`/`loop`/`break`/
  `continue`/`return`, `match` on enum-like values, `for` (post-lowering while),
  strings with `+` concatenation and f-string interpolation;
- builtins: `output` (append to `RunOutput.stdout`), `input` is an error,
  `string.len/contains/lower`, `list.len/add/extend`, `env.args`;
- runtime errors become `Diagnostic` (`E0600`) with the failing span;
- `run_source` is pure-ish: returns captured stdout, so tests need no process.

Target programs: `examples/hello-world` and `examples/fibonacci-n-step`.

## Acceptance (definition of done)
- `run_source` executes hello-world and returns exactly `Hello, world!\n`.
- Fibonacci prints the right value for n = 10 via a test.
- A division by zero / unknown function returns `Err`, never panics.
- Guard against runaway loops: >10M steps → error diagnostic.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`, `design/`,
  `examples/`, `jules/`, or `tests/golden/` (frozen).
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass.
- Do not leave `todo!()`, `unimplemented!()` or `panic!()` reachable from
  library code.
