# Task 28 — Make every example run (interpreter stdlib breadth)

Read `jules/README.md`, `design/03-memory-model.md` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/interp/eval.rs`, `src/interp/value.rs`, `src/interp/stdlib.rs` (new file)
- `tests/interp_examples.rs` (new file)

## Goal
`for f in examples/*/main.soma: soma run` must succeed for all 14 examples
(concurrency may exit after a bounded number of steps, but must not error).

Known gaps to close:
- `error-handling`: `invalid types for +` — desugared `n = if (true) { __err = call; if is_error(__err) { return __err }; __err }`
  must yield the inner value; verify if-expression values and `!` propagation.
- `rand.choice(list)` — deterministic pseudo-random pick (LCG seeded per run).
- `thread.async(f)` / `thread.wait(handle)` — run to completion synchronously, return the value.
- `net.http.Router` — object with `new(host port)`, `get(path handler)`, `listen()`; `listen` prints the routes and returns.
- `io.term.input(prompt)` — read one line from stdin; if stdin is empty/closed return the empty string instead of erroring.
- `guessing-game` — must run non-interactively: `input` returns "" on EOF, the game ends after a fixed number of rounds.

## Acceptance
- `tests/interp_examples.rs`: one test per example asserting `run_file` is `Ok`; snapshot stdout for `hello-world`, `fibonacci-n-step`, `pattern-matching`, `oop-shapes`.
- No `panic!` reachable; step limit still enforced.

## Hard rules
Only the files above. No scripts/scratch files. No new deps. `cargo test` must pass. No `todo!()`.
