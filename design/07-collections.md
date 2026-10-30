# 07 — Collections: how things grow without a heap

The question: *if there's no heap allocator, how does a list or a map grow?*

The answer (inherited from Goose's model): **grow-only data stacks — arenas with
a bump pointer — not `malloc`.**

## Growth by pointer bump

A "data stack" is a large reserved region of address space with a single
**bump pointer** marking the next free byte.

- **Growing a list = advance the pointer.** No free-list, no fragmentation, no
  per-element allocation.
- The reservation is *virtual*; the OS commits physical pages only as the bump
  pointer reaches them. So a list can "grow" to millions of elements cheaply.
- **Freeing = reset the pointer** when the scope ends — one constant-time store,
  however deep the structure nests.

```
for round in ..3
  scratch = []                  # a grow-only list on the data stack
  for i in 0..1000000
    scratch + i                 # bump pointer advances — sugar for .add()
  output(f"{scratch.len()}")    # 1,000,000
                                # scope ends: free = reset the pointer
```

## Why nothing moves

At most **one resizable value per stack, always at the top** — so growth never
has to relocate anything (there's nothing after it to shift). Consequence: a
reference into a growing list **stays valid while the list lives**, which a
relocating `Vec`/`vector` can't offer.

## A list

An array is laid out inline as `[metadata][elements…]` — contiguous, no pointers
to chase. Iteration streams through cache. Strings are the same thing (`u8`
arrays).

**The trade-off:** an array of *variable-size* elements is sequential — you can
iterate it but not randomly index it without a side index of references.

## A map / hash map

A hash map is a **grow-only array of buckets on the data stack** — same bump
growth, same scope-free. Rehashing = allocate a bigger bucket array higher on
the stack and re-fill; the old one is released with the scope, not individually
freed.

## What this means for soma

- Collections grow in **O(1) amortized** with no allocator cost.
- Bulk teardown is **O(1)** (scope-end pointer reset).
- The constraint (inherited): structures that must **outlive** their creating
  scope, or arbitrary pointer-graphs, need a different home (Goose uses a
  `reusable` pool with a freelist). **TBD for soma:** whether to adopt pools for
  long-lived data, and how transparent that is.

See [03-memory-model.md](03-memory-model.md) for the safety rules and
[01-philosophy.md](01-philosophy.md) for why soma also has to *hide* this
placement work that Goose makes the programmer do.
