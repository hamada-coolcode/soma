# soma

**A memory-safe, zero-runtime-cost, easy-to-use programming language.**

soma aims at the corner of the design triangle that's still mostly empty:

```
              Memory-safe
                 /\
                /  \
               /    \
              / soma \
             /        \
            /____________\
  Zero runtime        Easy to
     cost               use
```

- **Rust** is safe + zero-cost, but hard (borrow checker, lifetimes).
- **Zig** is easy + zero-cost, but not memory-safe.
- **Go** is safe + easy, but pays with GC + runtime cost.
- **Goose** is safe + fast, but makes *you* manage data layout and forbids
  escaping closures / dynamic dispatch.
- **soma** wants all three — and wants to *hide* the memory management that
  Goose puts on the programmer, while keeping the flexible features Goose omits.

## Taste

```soma
fn
  div a: int b: int -> MathError!int
    if b == 0
      return DivisionByZero
    return a / b

  main -> !
    output(f"10/2 = {div(10 2)!}")
    div(10 0) ! output("cannot divide by zero")
```

Indentation instead of braces. No commas in calls. No parens in declarations.
`!` as a return type and as error propagation/handling. Implicit references (no
`&`, no `*`). Copy-free returns. Structural (duck-typed) traits. Lists grow with
`seq + x`.

## Status

**Early design.** Syntax and semantics are being shaken out through the
examples; the compiler is at the "think it through before building it" stage.

## Layout

| Path | What |
| --- | --- |
| [`design/`](design/) | **The source of truth** — every language decision |
| [`examples/`](examples/) | Same programs in Rust / Go / Python / C++ / Zig / soma (Rosetta-style) |
| [`examples/all-features/`](examples/all-features/) | One file, every feature — for review |
| `src/` | Rust compiler bootstrap (LLVM via inkwell) |

Start at [`design/README.md`](design/README.md).

## The language in short

| Aspect | Decision |
| --- | --- |
| Surface | 2-space indentation (2/4/6/8 OK), no braces/semicolons |
| Typing | static + local inference, **structural traits** |
| Memory | **no GC / RC / destructors / lifetime annotations**; scope-based |
| References | **implicit — no `&`, no `*`** |
| Generics | monomorphized (`Stack[T]`, `input[uint]`) |
| Errors | values; `Error!Type`, `!` propagate, `! handler` |
| Concurrency | `#[async]` + list-comprehension + `wait`; no shared memory |
| Targets | Rust + LLVM first, self-host later |

Open decision (marked **TBD** in the docs): the enum design.
