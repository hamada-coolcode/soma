# 09 — CLI

The `soma` command drives everything. **TBD — command set is a proposal**, drawn
from what Go/Rust/Goose do, for review.

## Proposed commands

```
soma run <file.soma>            # compile + execute in one step
soma build <file.soma>          # compile to a native executable
soma lint <file.soma>          # type + memory analysis only (no codegen)
soma test [<file.soma>|<dir>]   # run `test` blocks
soma fmt [<file.soma>...]       # format source
soma lsp                        # language server (diagnostics as you type)
```

## Useful flags

```
soma run main.soma --emit-psoma     # dump the desugared .psoma IR (debug)
soma run main.soma --emit-ir        # dump LLVM IR
soma lint main.soma --verbose      # extra analysis detail
soma run main.soma -- args...       # pass arguments to the program
```

## Examples

```bash
# run the hello-world example
soma run examples/hello-world/main.soma

# just type-check the guessing game
soma lint examples/guessing-game/main.soma

# see what the preprocessor produced
soma run examples/fibonacci-n-step/main.soma --emit-psoma

# build an executable
soma build examples/minigrep/main.soma -o minigrep
./minigrep duct examples/minigrep/main.soma
```

## Exit codes (proposed)

| Code | Meaning |
| --- | --- |
| 0 | success |
| 1 | compile / analysis error |
| 2 | runtime error |
| 3 | usage / bad arguments |

## Program arguments

Arguments after `--` are available to the program through the standard `env`
module (`args()`), as in `examples/minigrep/`:

```
import
  lib:std
    env
      args
```

---

**Open questions for the CLI:**
- Single binary doing `run`/`build`/`lint`, or separate subcommands as above?
- Is `test` a subcommand or does `soma test` scan a directory by default?
- Do we want a package/module manager now, or defer it entirely for v1?
