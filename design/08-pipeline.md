# 08 — Compiler pipeline

soma is **written in Rust first**, targeting **LLVM**, before it self-hosts.

## The stages

```
main.soma  ──preprocessor──►  main.psoma  ──analysis──►  LLVM IR  ──►  binary
  (sugared,     (desugar)      (explicit,     (types,       (inkwell)
   indented)                    no indent,     scope &
                                semicolons)    memory)
```

### 1. Preprocessor (`.soma` → `.psoma`)

The high-level file is **sugared**: indentation, `f"…"` strings, `!`
propagation/handlers, lambdas, `for … in`, comprehensions, `#[async]`, no commas.
The preprocessor desugars all of it into an **explicit, regular** form:

- indentation → explicit blocks (braces)
- `f"{a + b}"` → concatenation of parts
- `expr!` / `expr ! handler` → explicit error match
- `for x in it` → iterator loop
- `[e for x in it]` → build-and-append loop
- `L + x` → `.add()` / `.extend()` (append sugar, see 02)
- commas restored; `.` for everything (no `::`)

`.psoma` is an **internal IR only — not a user-facing language.** It exists to be
read by the next stage; it can change freely. It's dumped with `--emit-psoma`
purely for debugging.

### 2. Analysis

- **Type check** (static, with inference; structural trait conformance).
- **Scope-based memory check** — every reference's *root* is inferred; a
  reference may not outlive its root. This replaces a Rust-style borrow checker
  (see [03-memory-model.md](03-memory-model.md)). No Polonius/lifetime
  annotations in the source.
- **Error check** — `!` propagation consistency: a body that propagates must
  declare a fallible return type.

### 3. Codegen (LLVM)

Lower the verified program to LLVM IR via **[inkwell]** and emit a native
executable.

## Why a preprocessor at all

Indentation-based, sugared syntax is pleasant to *write* but painful to *parse
and analyze*. Splitting the two means:

- the backend sees only a regular, explicit grammar (easy to parse, easy to
  extract facts from),
- indentation errors surface early, at the front line,
- the sugar layer can evolve without touching type/memory analysis.

## Tooling chosen

| Crate | Role |
| --- | --- |
| [inkwell] | LLVM bindings — codegen |
| [chumsky] | parser combinators |
| [ariadne] | pretty diagnostics with source spans |

**Source spans** are threaded through every stage so an error in `.psoma`
analysis maps back to the `.soma` line the user actually wrote. Design this in
from stage 1 — retrofitting it later is miserable.

## Bootstrap order (planned)

1. Lexer with `INDENT`/`DEDENT`.
2. Parser → AST (regular grammar once indentation is resolved).
3. Desugar passes → `.psoma`.
4. Type check + scope/memory analysis on `.psoma`.
5. Span mapping back to `.soma` for diagnostics.
6. LLVM codegen — *start by transpiling `.psoma` to C or Rust as a stopgap, then
   swap to inkwell directly.*
7. Self-host: rewrite the compiler in soma.

## Self-hosting

Once soma can compile itself, the compiler is rewritten in soma targeting LLVM
(e.g. via Cranelift or LLVM bindings), retiring the Rust bootstrap.
