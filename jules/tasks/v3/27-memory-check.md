# Task 27 — Memory analysis: moves and ownership

Read `jules/README.md` (rules) and `design/03-memory-model.md` first. The design
is the spec; this task makes it enforceable.

## Files you may touch (ONLY these)
- `src/mem/mod.rs`
- `tests/mem.rs`

## Goal
A conservative, purely syntactic move/ownership pass over parsed items,
returning diagnostics. No AST rewriting, no CLI changes.

Rules to enforce:
1. **Copy types** — `int`, `uint`, `float`, `bool`, `null` are copied on
   assignment/argument passing; using them again is always fine.
2. **Move types** — `object` instances, lists (`T[]`), and unknown/`T`
   bindings move when used as a value: assigning, passing as an argument, or
   returning transfers ownership.
3. **Use after move** → `E0901`, message names the binding and the move site's
   line (`"`x` used after move (moved at line 12)"`).
4. **Double move** on the same path → `E0902`.
5. **Reassignment revives** a binding: `x = ...` clears its moved state.
6. **Branch merge is conservative**: a binding counts as moved after an
   `if`/`elif`/`else` only if moved on *every* branch; inside a loop body,
   moved iff moved before the loop or on all paths through the body.
7. **Early return/break/continue** terminate the path — do not report a use
   after them (that is task 19's unreachable-code warning, not a move error).
8. **Params**: assigning to a param is `E0301` (resolver owns it) — do not
   duplicate that diagnostic. Moving a param out is allowed.
9. `_`-prefixed bindings are exempt from `E0901` but still tracked for `E0902`.

## Acceptance (definition of done)
- `tests/mem.rs` with hand-built AST fixtures for each rule: copy reuse ok,
  move ok, use-after-move, double move, revive by reassignment, branch-merge
  (one branch moves → ok; both move → moved), loop body, return-then-use not a
  move error, param read (`E0301` not emitted here).
- `examples/` corpus yields **zero** `E09xx` diagnostics (add a test that walks
  `examples/*/main.soma` and asserts no memory diagnostics).

## Hard rules
- Only the files listed above. Never edit `Cargo.toml`, any `mod.rs`,
  `design/`, `examples/`, `jules/`, or `tests/golden/`.
- Keep `pub fn check(&[Item]) -> Vec<Diagnostic>`. No new dependencies.
- `cargo test` must pass. No `todo!()`/`unimplemented!()` in library code.
