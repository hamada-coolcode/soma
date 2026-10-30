# soma — design notes

> A memory-safe, zero-runtime-cost, easy-to-use systems language.
> This folder is the source of truth for every language decision.

soma aims at the corner of the design triangle that everyone wants and nobody
has landed in yet:

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

- **Rust** picks safety + zero-cost, and pays for it with borrow-checker pain.
- **Zig** picks ease + zero-cost, and pays for it with manual memory.
- **Go** picks safety + ease, and pays for it with GC + runtime cost.
- **Goose** picks safety + zero-cost + speed, and pays for it by making *you*
  think about data layout and forbidding escaping closures / dynamic dispatch.
- **soma** aims at all three: safety + zero-cost, *without* making you think
  about memory, and *without* giving up the flexible features Goose omits.

## Index

| Doc | What it covers |
| --- | --- |
| [01-philosophy.md](01-philosophy.md) | Goals, the triangle, how soma differs from Rust & Goose |
| [02-syntax.md](02-syntax.md) | Indentation, keywords, signatures (`->`), operators, sugar |
| [03-memory-model.md](03-memory-model.md) | Implicit references (no `*`), read-only params, move returns, scope-based memory |
| [04-type-system.md](04-type-system.md) | Static + inference, structural traits, generics, optionals |
| [05-errors.md](05-errors.md) | `!` propagation, `!` handlers, `error` blocks |
| [06-concurrency.md](06-concurrency.md) | `#[async]` + `wait`, no shared memory |
| [07-collections.md](07-collections.md) | How lists, strings, maps grow without a heap |
| [08-pipeline.md](08-pipeline.md) | Preprocessor → `.psoma` → analysis → LLVM |
| [09-cli.md](09-cli.md) | Command-line usage |

The [../examples/](../examples/) folder holds the same small programs written in
Rust, Go, Python, C++, Zig and soma — a Rosetta-Code-style suite used to design
each paradigm before committing to soma's form.

## Status

Early design. Syntax and semantics are being shaken out through the examples.
Anything marked **TBD** is an open decision, not a settled rule.
