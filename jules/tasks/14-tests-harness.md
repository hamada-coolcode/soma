# Task 14 — examples corpus harness

Read `design/02-syntax.md` and look at every `examples/*/main.soma`.

## Files you may touch (ONLY these)
- `tests/examples_corpus.rs` (create it)

## Goal
A dependency-free integration test that walks `examples/**/*.soma`
(use `std::fs`, relative to `CARGO_MANIFEST_DIR`) and enforces the locked
surface rules **with plain string analysis** — no lexer/parser calls (those
land in parallel):
1. indent is spaces only (no tab characters in indentation)
2. the file's indent width (first indented line) is 2, 4, 6 or 8, and every
   non-blank line's indentation is a multiple of it
3. no trailing whitespace on any line
4. file ends with exactly one newline
5. no `::` anywhere
6. no `struct` keyword (must be `object`)
7. no `main!` / `main !` (fallible entry is `main -> !`)
8. no `<>` generic brackets (generics use `[]`)
9. at least 14 example directories exist, each with a `main.soma`
10. every file that imports `output` also uses the `import` tree form (i.e. if
    "output(" appears and no `import` line exists, fail)

Failures must name the file and line number.

## Tests
The harness itself is a `#[test]`; additionally unit-test the helper functions
(indent width detection, tab detection, trailing whitespace) with `include_str!`
tiny inline fixtures. Ensure the whole suite passes against the current tree —
if a rule already fails, fix ONLY your test's interpretation and report the
offending file in the PR description (do not edit examples/).

## Hard rules
Only create/edit `tests/examples_corpus.rs`. Never edit `src/`, `Cargo.toml`,
`design/`, `examples/`, `jules/`. No new deps.
