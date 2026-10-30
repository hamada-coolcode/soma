# Task 26 — C backend (`soma build --emit-c`)

Read `jules/README.md` (rules), `design/08-pipeline.md` (codegen section) and
`jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/codegen/mod.rs`
- `src/codegen/c.rs`
- `tests/codegen.rs`

## Goal
Turn **lowered** items (`desugar::lower` output) into one self-contained C
translation unit. This is the documented stopgap before LLVM, not a toy.

Scope for this task — must work end to end:
- `main -> !` becomes `int main(void)`;
- int/uint/float/bool/string literals; `+ - * / %`, comparisons, `and`/`or`/`not`
  map to C operators (integer division must truncate like soma);
- local bindings, assignment, `return`, `if`/`elif`/`else`, `while`, `for`;
- function definitions and calls (soma signatures → C prototypes emitted before
  definitions so declaration order doesn't matter);
- `object` → `typedef struct`; field access `.`, method calls via `recv.m(args)`
  (generate `m` as a free function taking the receiver pointer);
- `io.term.output(x)` → `fputs`/`printf`, `io.term.input()` → `getline` on stdin;
- string values are `const char*`; `+` on strings concatenates (arena-allocate,
  never free);
- a small fixed prelude (arena bump allocator, `soma_str_concat`,
  `soma_int_to_str`) emitted at the top of the unit.

Everything unsupported (closures, generics instantiation, async, error
unwinding) must emit a `/* TODO(task NN): ... */` comment **and** return an
error from the public API rather than silently printing nothing.

Public API (keep these signatures):
- `codegen::emit_c(&[Item]) -> String` — full unit, C99, compiles with
  `cc -std=c99 -O2 -w`.
- add `codegen::emit_c_checked(&[Item]) -> Result<String, Diagnostic>` — first
  unsupported construct becomes one `E1001` diagnostic naming the construct and
  its span.

## Acceptance (definition of done)
- `tests/codegen.rs` asserts exact C text for `hello`, arithmetic and an
  `object` fixture (snapshot strings, whitespace-normalised).
- If `cc` is on `PATH`: compile + run `tests/golden/hello.soma` and
  `examples/arith/main.soma` equivalents and compare stdout; `#[ignore]` the
  test when `cc` is missing (`which cc` check at runtime, not a build dep).
- No panic on an empty item list; empty input yields a unit with just the
  prelude and an empty `main`.

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs` outside
  `src/codegen/`, `design/`, `examples/`, `jules/`, or `tests/golden/`.
- No new dependencies (no inkwell/LLVM). Pure string emission.
- `cargo test` must pass. No `todo!()`/`unimplemented!()` in library code.
