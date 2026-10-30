# 03 — Memory model

**One line:** no garbage collector, no reference counting, no destructors, no
lifetime annotations — and **no `*` operator**. References are implicit; the
compiler keeps them safe.

soma takes its inspiration from [Goose](https://aardappel.github.io/goose)'s
scope-based memory rather than Rust's borrow checker.

## The rules

1. **No heap allocator / GC / RC / destructors.** Data lives in compiler-managed
   **scopes** (arena/"data-stack" style). Memory is released when its scope ends
   — freeing a deeply-nested structure is a constant-time operation.
2. **References are implicit.** You write `hello`, not `&hello` or `*hello`.
   Passing a value to a function, returning it, storing it, closing over it —
   none of these need a marker. There is no copy-vs-reference choice for the
   programmer to make.
3. **Values move out on return.** `return seq` hands ownership of `seq` to the
   caller; the local dies with the callee's scope. No copy, no dangling.
   Multiple returns are allowed — e.g. a function can return two locals while
   the rest of its frame is released.
4. **The compiler infers reference roots.** Every reference has a *root* — the
   variable that bounds its lifetime. A reference may not outlive its root. The
   compiler checks this and rejects escapes; **no annotations are written.**
5. **Reads never copy.** Passing data to be read is always by reference.

## Mutation — settled

Because there is no `*`, a parameter *is* a reference to the caller's value — so
the rules about who may write are what keep calls side-effect-free:

1. **Locals are mutable.** `i = i + 1`, `results + line` on your own variable.
2. **Parameters are read-only.** Assigning to a parameter name is a compile
   error, and so is mutating *through* one (`param.add(x)`, `param.field = v`).
   This is the check that kills the classic footgun:
   ```
   fn add_one x: int -> int
     x = x + 1          # ✗ error: parameters are read-only
     return x + 1       # ✓ reads are fine

   fn main
     m = 5
     output(f"{add_one(m)}")   # m cannot have been changed
   ```
   To hand a result back, **return it** — returns move, so it costs nothing.
3. **`self` is the exception.** Methods may modify their receiver — that's what
   objects are for (`st.push(1)` changes the caller's `st`). At the call site
   the receiver binding must be mutable: a mutating method on a read-only
   parameter is an error.
4. **The compiler checks callee bodies.** A function is classified *mutating*
   if its body writes through `self`; calling it on a read-only binding is
   rejected.

The caller's data is never silently changed by a call. The only writes that
land on it come through `self` on a binding you own.

## Why not Polonius / a borrow checker?

soma *could* have reused Rust's Polonius engine for lifetime checking. The
scope-based model was chosen instead because:

- It removes lifetime inference — the single hardest part of a Rust-like
  language.
- It matches the "easy" goal: there is nothing for the user to encode.
- It trades some flexibility (self-referential / long-lived heap structures)
  for simplicity — an accepted cost for soma's target programs.

## What this costs

- Less suited to arbitrary pointer-graphs and structures that outlive their
  creation scope (Goose's known limitation).

See [07-collections.md](07-collections.md) for how lists/maps grow under this
model, and [08-pipeline.md](08-pipeline.md) for how it's enforced at compile
time.
