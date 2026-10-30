# 01 — Philosophy

## The three goals

soma is built to hold three properties at once. Each is easy alone; the trick
is keeping all three:

1. **Memory safe** — no dangling pointers, no use-after-free, no data races,
   by construction. The compiler proves it.
2. **Zero runtime cost** — no garbage collector, no reference counting, no
   destructors, no runtime bookkeeping for memory. What you write is what runs.
3. **Easy** — indentation syntax, no lifetime annotations, no `&`/`&mut`/`*`
   markers, implicit references, familiar high-level sugar.

## What soma is *not*

- **Not a Rust killer by intent.** Rust remains the reference for safe,
  zero-cost systems programming. soma is a different point on the usability
  axis, not a replacement for Rust's ecosystem. Although it could be used as a Rust-killer also.
- **Not a GC language.** No runtime memory management at all.
- **Not dynamic.** The indentation and sugar are a *surface*; underneath soma
  is statically typed with a real type checker.

## How soma differs from Rust

| | Rust | soma |
| --- | --- | --- |
| Safety mechanism | borrow checker + lifetime annotations | scope-based memory (see 03) |
| Passing to a function | move or borrow (you choose / infer) | **reference** by default — implicit, no `*` |
| Reference syntax | `&x`, `&mut x` everywhere | none — references are implicit |
| Surface syntax | braces + semicolons | **indentation**, no braces/semicolons |
| Interfaces | explicit `impl Trait for Type` | **structural** (duck-typed) traits |
| Errors | `Result<T,E>` + `?` | `Error!Type` + `!` |

The headline difference: Rust asks you to encode ownership with `&`/`&mut`/`<'a>`.
soma asks for nothing — references are implicit and the compiler keeps them safe.

## How soma differs from Goose

Goose (by Wouter van Oortmerssen) is the closest living cousin: memory-safe,
no heap/GC/lifetimes, scope-freed inline data, compiled to C, extremely fast.
soma takes Goose's memory-model *inspiration* but draws the line differently:

| Goose | soma |
| --- | --- |
| C/Rust-like braces, sigils (`.=`, `[>..]`, `&<u32>`) | **indentation**, `object`/`fn`/`trait` blocks |
| *"You have to think about where data lives"* | **the compiler manages placement** for you |
| No escaping closures, no function pointers | closures are **first-class values** |
| No dynamic dispatch beyond ADT tags | **structural traits** with dynamic dispatch |
| Variable-size arrays: iterate but not index | **indexable** collections |
| Recursive functions can't keep growable data | recursion **with** state |
| Whole-program compilation only | **separate compilation** / real libraries |

**Positioning in one line:** Goose is *peak speed, you do the layout thinking,
and you accept its constraints.* soma is *the compiler does the thinking, and
the features Goose forbids all work.* soma targets the real-world programs
Goose makes awkward — and if a benchmark ever comes up short, that's what
optimizations are for.

## Non-goals

- Runtime reflection, dynamic loading, `eval`.
- A large feature surface in v1. Ship a small, coherent core; grow deliberately.
