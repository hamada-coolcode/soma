# Task 33 — `soma repl`

Read `jules/README.md`, `design/09-cli.md` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/repl.rs` (new file)
- `tests/repl.rs` (new file)

## Goal
A line-oriented REPL over the existing interpreter:
- `repl::run_session(input: &str) -> String` — reads one statement/expression per
  line from `input`, evaluates it, appends `=> <value>` (or the error) to the output;
- definitions (`fn`, `object`, `const`, `error`, `enum`) persist across lines;
- `:quit` ends the session; `:reset` clears definitions;
- blank lines and `#` comments are ignored;
- never panics: a parse error prints the diagnostic and continues.

This is a pure function (input string in, transcript out) so it is testable
without a TTY. CLI wiring (`soma repl`) is done by the integrator.

## Acceptance
- `tests/repl.rs`: arithmetic across lines using a variable defined earlier, a
  function defined then called, a parse error followed by a successful line,
  `:reset` clearing state, `:quit` stopping consumption.
- Transcript format stable and asserted.

## Hard rules
Only the two files above. No new deps. `cargo test` must pass. No `todo!()`.
