# Task 32 — IR optimiser: constant folding + dead code elimination

Read `jules/README.md`, `design/08-pipeline.md` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/opt/mod.rs` (new file)
- `src/opt/fold.rs` (new file)
- `src/opt/dce.rs` (new file)
- `tests/opt.rs` (new file)

## Goal
A small, semantics-preserving optimiser over the **lowered** AST:
- `opt::fold(&mut [Item])` — fold integer/float/bool constant expressions
  (`2 * 3` → `6`, `not false` → `true`, `"a" + "b"` → `"ab"`); never fold division
  by zero; never fold across a call;
- `opt::dce(&mut [Item])` — delete unreachable statements after `return`/`break`/
  `continue`, and `if false`/`if true` branches (keeping the taken branch);
- `opt::run(&mut [Item]) -> OptReport { folded: usize, removed: usize }` applies
  both to a fixed point (max 8 rounds);
- the pass must not change observable behaviour: `soma run` output is identical
  with and without it on every example.

## Acceptance
- `tests/opt.rs`: fold/dce unit cases, idempotence (`run` twice → no changes on
  the second pass), and a test that running every `examples/*/main.soma` through
  the optimiser then the interpreter gives the same stdout as without.
- No panics on empty input.

## Hard rules
Only the four files above. No new deps. `cargo test` must pass. No `todo!()`.
