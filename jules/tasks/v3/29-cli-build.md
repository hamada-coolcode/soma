# Task 29 — Wire `soma build`, `soma fmt`, `soma test`, `--json`

Read `jules/README.md`, `design/09-cli.md` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/cli.rs`
- `tests/cli_build.rs` (new file)

## Goal
Give the binary the commands the library already implements:
- `soma fmt FILE [--check]` → `fmt::format_items`; `--check` exits 1 when the file is not canonically formatted, 0 when it is.
- `soma test FILE` → `testrun::run_tests`; prints the transcript, exits 1 if any test failed.
- `soma build FILE [-o OUT] [--emit-c]` → `codegen::emit_c`. With `--emit-c`, write OUT (default `FILE.c`). Without it, run `cc -std=c99 -O2 -w -o OUT FILE.c` via `std::process::Command`; a missing `cc` is a clear error, not a panic.
- `soma parse FILE --json` → `emit_json::emit_json`.
- Exit codes stay 0/1/2 as documented.

## Acceptance
- `tests/cli_build.rs`: `fmt --check` on a formatted file exits 0 and on an unformatted one exits 1; `parse --json` output starts with `[`; `build --emit-c` writes a file containing `int main`; `test` on a fixture with one failing test exits 1.
- Existing `tests/cli.rs` must stay green.

## Hard rules
Only the files above. Do not modify the modules you call. No new deps. `cargo test` must pass.
