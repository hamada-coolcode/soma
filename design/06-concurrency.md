# 06 — Concurrency

soma's concurrency model: **mark a function `#[async]`, call it to launch it,
collect the handles, and `wait` for results.** No shared mutable memory.

## The shape

```
import
  lib:std
    thread
      async
      wait

fn
  #[async]
  worker id: int -> int
    sum = 0
    j = 0
    while j < 1000000
      sum = sum + j
      j = j + 1
    output(f"worker {id} sum = {sum}")
    return sum

  main
    handles = [worker(i) for i in ..4]   # launch 4 tasks, collect handles

    total = 0
    for h in handles
      total = total + wait(h)             # wait for each result

    output(f"total = {total}")
```

## Rules

1. **`#[async]` marks a task.** The attribute tells the compiler this function
   runs concurrently.
2. **Calling it launches it.** `worker(i)` starts the task and returns a
   *handle* — you don't write a separate `spawn`.
3. **List comprehension collects handles:** `[worker(i) for i in ..4]` builds
   a list of in-flight tasks.
4. **`wait(h)` blocks for one result.** Reduce the handles to a value.
5. **No shared mutable memory.** Each task computes with its own locals and
   returns its result through `wait`. There is no shared counter, no lock, no
   atomics for you to reason about.

## Why no shared memory

Following Goose's stance: **data races, locks and memory-ordering bugs are
eliminated by not having shared mutable state in the language.** Tasks hand
results back through their return values. Because values are flat and
scope-managed, passing a result across is cheap (a contiguous copy), yet still
race-free.

## Relationship to the memory model

Because a task's inputs are references under the same scope rules (see
[03-memory-model.md](03-memory-model.md)), a task may not keep a reference
past the life of its root — the compiler enforces this like any other escape.
Returning a *value* from a task is always fine (it moves to the waiter).

**Open:** confirm whether cross-task values are copied at the boundary (Goose
style: flat value `memcpy`) or ownership-transferred. Either is race-free; the
choice affects what a task may return.
