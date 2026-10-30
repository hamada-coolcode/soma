# all-features

One file, every settled soma feature, so you can check the design reads right.

Run-through of what it demonstrates and the rule each one follows:

| Feature | Where | Rule |
| --- | --- | --- |
| Indentation (2-space) | whole file | blocks by indent, no braces/semicolons |
| `import` tree | top | `lib:std / io / term / output`, mirrors module path |
| `const` | `APP`, `SECRET` | named constants; `choice(1..100)` range |
| `error` block | `AppError` | declares error variants |
| `trait` + structural | `Shape`, `Triangle` | Triangle never declares it's a Shape |
| `object` (not `struct`) | `Circle`, `Stack[T]`, `Node` | `object` keyword |
| generic object | `Stack[T]`, `Stack.new[int]()` | generics in `[]`, monomorphized |
| optional `T?` | `Node.next`, `null` | absent value is `null` |
| `enum` + `match` | `Expr`, `eval` | **TBD** |
| signature `name params -> returnType` | all `fn`s | **bare params, no parens, no commas**, return after `->` |
| `Error!Type` return | `div a b -> AppError!int` | can fail, returns `int` |
| `main -> !` | entry | **`!` is the return type** — can fail, returns nothing |
| no commas in calls | `add(a b)`, `combined_area(c t)` | space-separated args |
| `.` not `::` | `Stack.new[int]()`, `st.peek()` | one member operator |
| `f"..."` strings | everywhere | `{expr}` interpolation |
| lambdas | `x => x * 3`, `(a b) => b - a` | single / multi-arg |
| higher-order | `apply x: int f: (int) int -> int` | function type as a param |
| named ctor args | `Circle(radius = 2.0)`, `Node(val = 1 next = null)` | `field = value`, no commas |
| `for … in`, `while`, `loop`/`break` | `main -> !` | iteration |
| `if / elif / else` + `=>` one-liner | `for x in [1 2 3]` | single-line bodies |
| range `..b` default, `0..b` explicit | `0..n`, `1..100` | `..b` starts at 1 unless you need 0 |
| list `+` append/extend | `out + i` in `range` | sugar for `.add()`, in place, aliases left |
| list comprehension | `[worker(i) for i in ..4]` | `[expr for x in iter]` |
| `!` propagate | `st.pop()!`, `div(10 2)!` | hand error to caller |
| `! handler` | `div(10 0) ! output(...)` | run code only on failure |
| `#[async]` attribute | `worker` | marks a concurrent task |
| `async` / `wait` | concurrency block | launch via call, `wait(h)` for result |
| move-return | `range` returns `out` | value moves to caller, **no `*`** |
| read-only params | every `fn` | assigning to a param = error; return instead |
| `test` block | bottom of file | `"name"` + body, last expression is the assertion |
| arithmetic `*` | `0.5 * base`, `x * 2` | multiplication (no reference `*` exists) |

## Not present / open

- **No `*` operator anywhere** — references are implicit (settled).
- **Parameters are read-only**, `self` is mutable, returns move (settled —
  see [design/03-memory-model.md](../../design/03-memory-model.md)).
- **Enums** — form is being reworked; `Expr`/`match` is a placeholder. **TBD.**
- Error paths in `main -> !` aren't caught at the top level — fine for a tour.
