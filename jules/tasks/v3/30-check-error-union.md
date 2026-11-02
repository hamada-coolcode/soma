# Task 30 — Error-union typing (`T!U`, `!` propagation)

Read `jules/README.md`, `design/05-errors.md` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/types/check_error.rs` (new file)
- `tests/check_error.rs` (new file)

## Goal
Type error unions and `!` propagation:
- `MathError!int` is a union of the error set and `int`; a bare `int` return from such a function is fine, and so is returning a declared variant.
- returning an undeclared variant from a function with an error union is `E0601`, naming the variant and the allowed set;
- `expr!` where `expr` is not an error union is `E0602`;
- `expr!` in a function whose return type cannot carry that error is `E0603`;
- `expr! handler` yields the non-error type of `expr` and type-checks `handler` against the error set;
- `T?` (optional) is distinct from an error union: using `?` on an error union is `E0604` and vice versa.

## Acceptance
- Hand-built AST + `TypeTable` fixtures for each rule, plus one positive case per form.
- Uses `unify` only; no AST changes.

## Hard rules
Only the two files above. No new deps. `cargo test` must pass. No `todo!()`.
