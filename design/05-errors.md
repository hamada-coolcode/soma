# 05 — Errors

Errors are **values**, not exceptions. There is no stack unwinding, no `throw`,
no `Result<T,E>` boilerplate. The glyph `!` carries the whole system.

## Declaring errors

An `error` block declares an error type and its variants:

```
error
  MathError
    DivisionByZero
```

## Return type: `Error!Type`

A fallible function names its error and success types, joined by `!`:

```
div a: int b: int -> MathError!int
  if b == 0
    return DivisionByZero     # return an error variant
  return a / b                # return a success value
```

### `!` as the whole return type

`main -> !` and `run config: Config -> !` write `!` as the **entire** return
type. This means: *the function can fail and returns no success value* — it only
propagates errors upward. So:

| Signature | Meaning |
| --- | --- |
| `main` | returns nothing, cannot fail |
| `main -> !` | returns nothing, **can fail** (propagates) |
| `add a b -> int` | returns `int`, cannot fail |
| `div a b -> MathError!int` | returns `int`, can fail with `MathError` |
| `build args -> ConfigBuildError!Self` | returns `Self`, can fail |

If a function body uses error propagation (`!` on a call), its signature must be
marked fallible (`!` or `Error!Type`).

## `!` on a call — propagate

With nothing after it, `!` means: *if this call failed, hand the error to my
caller; otherwise give me the success value.*

```
main ->
  result = div(a b)!     # propagate: on failure main fails too
  output(f"{result}")
```

So `expr!` evaluates to the success value, or short-circuits the current
function with the error.

## `! handler` — catch

With code after it, `!` runs that code **only when the call fails**:

```
main -> !
  safe_div(total 0) ! output("cannot divide by zero")
  parse_and_sum(bad) ! output("parse failed")

  # guess failure exits the loop instead of propagating
  loop
    guess = input[uint]("Your guess: ")! break
```

This is the whole error toolkit: `!` alone propagates; `! <code>` handles.

## Design notes

- One glyph, one idea — "what to do about an error." Empty handler = propagate;
  non-empty = recover inline.
- Because errors are values flowing through signatures, there's no unwinder in
  the runtime: cheaper and more predictable than exceptions.
- The `!` return-type form mirrors Rust's `?` but reads as a type, matching
  soma's "return type after `()`" convention.
