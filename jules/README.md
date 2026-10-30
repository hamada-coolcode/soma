# jules — tasks for the Jules agent

This folder is the contract for autonomous sessions. Jules clones this repo at
HEAD, reads a brief, implements it, runs `cargo test`, and opens a PR.

## Rules of engagement (every brief repeats these)

1. **File ownership.** A task may only touch the files its brief lists. Shared
   files (`Cargo.toml`, every `mod.rs`, `design/`, `examples/`, `jules/`) are
   off-limits unless a brief says otherwise.
2. **Keep the stub API.** Signatures in the stubs are the interface other tasks
   build on. Change bodies, not signatures.
3. **Tests are the contract.** Every task adds `#[cfg(test)]` unit tests in its
   own file(s), using hand-built fixtures (`Token::ident(..)`, `Token::op(..)`,
   AST literals). Do **not** depend on another module whose `TODO(task NN)`
   marks it unimplemented — those land in parallel.
4. **No new dependencies.** `cargo test` must pass. No LLVM, no codegen.
5. **Spec wins over this file.** If `design/*.md` and `jules/grammar.ebnf`
   disagree, follow the design doc and note it in the PR.

## Task index

| # | Brief | Owns |
| --- | --- | --- |
| 01 | [01-lexer-core.md](tasks/01-lexer-core.md) | `src/lexer/scan.rs`, `src/lexer/indent.rs` |
| 02 | [02-lexer-literals.md](tasks/02-lexer-literals.md) | `src/lexer/literal.rs` |
| 03 | [03-lexer-symbols.md](tasks/03-lexer-symbols.md) | `src/lexer/symbol.rs` |
| 04 | [04-diag-sourcemap.md](tasks/04-diag-sourcemap.md) | `src/diag/sourcemap.rs` |
| 05 | [05-diag-render.md](tasks/05-diag-render.md) | `src/diag/render.rs`, `src/diag/error.rs` |
| 06 | [06-parse-blocks.md](tasks/06-parse-blocks.md) | `src/parse/blocks.rs` |
| 07 | [07-parse-signature.md](tasks/07-parse-signature.md) | `src/parse/signature.rs` |
| 08 | [08-parse-types.md](tasks/08-parse-types.md) | `src/parse/types.rs` |
| 09 | [09-parse-expr.md](tasks/09-parse-expr.md) | `src/parse/expr.rs` |
| 10 | [10-desugar-append.md](tasks/10-desugar-append.md) | `src/desugar/append.rs` |
| 11 | [11-desugar-errors.md](tasks/11-desugar-errors.md) | `src/desugar/errors.rs` |
| 12 | [12-desugar-comprehension.md](tasks/12-desugar-comprehension.md) | `src/desugar/comprehension.rs` |
| 13 | [13-desugar-psoma.md](tasks/13-desugar-psoma.md) | `src/desugar/psoma.rs` |
| 14 | [14-tests-harness.md](tasks/14-tests-harness.md) | `tests/` |

Tasks 01–13 are one pipeline; they were written to be **file-disjoint** so they
can run simultaneously. Integration (end-to-end on `examples/`) is a later task.

## Launching

```bash
jules remote new --repo hamada-coolcode/soma \
  --session "Read jules/tasks/01-lexer-core.md. Follow jules/README.md rules. Only edit the listed files. cargo test must pass."
```

Monitor with `jules remote list --session`, pull with
`jules remote pull --session <id> [--apply]`.
