# 04 — Type system

## Static with local inference

soma is **statically typed**: every expression's type is known before the
program runs, and mismatches are compile errors. The indentation and sugar are
a *surface* only.

Local `let`-free assignment infers from the right-hand side:
```
a = 17            # int
name = "hamada"   # string
seq = []          # inferred from first use / annotation
```

**Structural traits** mean the checker matches *shape*, not declared names —
the type system can verify a `Triangle` is a `Shape` purely because it has
`area()` and `name()`.

## Structural traits (duck typing)

A trait declares a required method shape; **any object that happens to match it
qualifies — it never has to declare the relationship.**

```
trait
  Shape
    area -> float
    name -> string

object
  Circle
    radius: float
    area -> float
      return pi * self.radius * self.radius
    name -> string
      return "circle"

  Triangle      # never says "I am a Shape" anywhere
    base: float
    height: float
    area -> float ...
    name -> string ...

fn
  combined_area s1: Shape s2: Shape -> float
    return s1.area() + s2.area()

  main
    combined_area(Circle(radius = 2.0) Triangle(base = 6.0 height = 4.0))
```

Conformance is checked at compile time (no runtime cost), but the *rule* is
Go-style implicit/structural — closest to Go's interfaces, not Rust's explicit
`impl`.

## Generics — monomorphized

Generic parameters live in `[]` and are **specialized at compile time** (one
concrete copy per instantiation — like Rust/Zig/Go), so there is no runtime
overhead.

```
object
  Stack[T]
    items: T[]
    push item: T ...
    pop -> StackError!T ...

fn
  main -> !
    s = Stack.new[int]()      # Stack[int]
    w = Stack.new[string]()   # Stack[string] — a separate type
```

Generic calls: `input[uint]("...")`, `v.parse[int]()!`.

**Type arguments are inferred** from arguments where possible; otherwise written
explicitly as above.

## Optionals

`T?` is a value that may be absent; the absent value is `null`. It must be
checked before use.

```
object
  Node
    val: int
    next: Node?

fn
  main
    n = Node(val = 1 next = null)
    while n != null
      ...
      n = n.next
```

## Enums — TBD

Algebraic enums with `match` exist in draft form (see
`examples/pattern-matching/`):

```
enum
  Expr
    Num(int)
    Add(Expr Expr)

fn
  eval e: Expr -> int
    match e
      Num(v) => return v
      ...
```

**Open:** the enum design is being reworked. Alternatives under consideration:
tagged unions à la Goose (fixed-mode / variable-mode), or a different construct
altogether. Nothing here is settled.

## Arrays

- `T[]` — **growable list** (`int[]`, `string[]`, `T[]`).
- `T[n]` — **fixed-size array**, `n` known at compile time (`int[10]`).
