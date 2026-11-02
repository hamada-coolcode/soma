# Task 31 — Structural trait checking

Read `jules/README.md`, `design/04-type-system.md` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/types/check_trait.rs` (new file)
- `tests/check_trait.rs` (new file)

## Goal
soma traits are **structural** — an object satisfies a trait if it has the methods,
with no `implements` clause.
- passing an object to a `Shape` parameter requires every method name, arity and
  signature to match (`E0701`, message lists the missing/mismatched members);
- returning an object from a `-> Shape` function follows the same rule;
- a trait parameter called with a primitive or list is `E0702` unless the trait is
  empty;
- method signature comparison ignores parameter names, compares types and return type;
- report every missing member, not just the first.

## Acceptance
- Fixtures: object with all members (ok), one missing method, wrong arity, wrong
  parameter type, wrong return type, primitive passed, empty trait (ok).
- No AST changes; operate on definitions + `TypeTable`.

## Hard rules
Only the two files above. No new deps. `cargo test` must pass. No `todo!()`.
