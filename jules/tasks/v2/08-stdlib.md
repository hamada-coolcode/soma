# Task 08 — stdlib surface + builtin registry

Read `jules/README.md` (rules), `jules/grammar.ebnf` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `lib/std/**` (create)
- `src/builtin.rs`

## Goal
Write the standard library's **surface** (signatures only; bodies are not soma
yet) and register the builtins the examples need:

- `lib/std/io/term.soma`: `output s: string -> !`, `input[T] prompt: string -> T!`
- `lib/std/fs.soma`: `File` object with `path: string`, `read -> string[]!`,
  `write s: string -> !`, `exists -> bool`
- `lib/std/string.soma`: `len -> int`, `contains s: string -> bool`,
  `lower -> string`, `split sep: string -> string[]`
- `lib/std/list.soma`: `len -> int`, `add x: T -> !`, `extend xs: T[] -> !`,
  `map f: (T) T -> T[]`
- `lib/std/env.soma`: `args -> string[]`, `BoolVar name: string -> bool`

`builtin.rs`: fill `BUILTINS` with `path` + `sig` (soma type syntax) for every
function above, and make `type_from_str` reject non-types. Keep `find`.

## Acceptance (definition of done)
- Every `BUILTINS` entry resolves through `type_from_str` and `find`.
- A test asserts the set of paths is exactly the documented list (no extras).
- Files use the locked surface syntax and pass the examples-corpus style rules.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`, `design/`,
  `examples/`, `jules/`, or `tests/golden/` (frozen).
- Keep every existing `pub fn` signature. Additive changes only.
- No new dependencies. `cargo test` must pass.
- Do not leave `todo!()`, `unimplemented!()` or `panic!()` reachable from
  library code.
