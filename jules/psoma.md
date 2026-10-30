# `.psoma` — the lowered IR contract

`.psoma` is the output of `soma parse FILE --emit-psoma`: the AST after
`desugar::lower`, printed unambiguously. It is a **debug and test format**, not
a documented language. Golden files in `tests/golden/` are **frozen**: a task may
make its code match them, never the other way round.

## Printing rules (deterministic — byte-exact)

- UTF-8, `\n` newlines, **exactly one trailing newline**. Empty program → `""`.
- Indentation is 2 spaces per level. It carries no meaning; braces do.
- One blank line between top-level items. **No blank lines inside bodies.**
- Every statement ends with `;`. Every block is `{ ... }` on the same line as
  its header, closing brace on its own line at the parent's indent.

### Items

```
import lib:std::io::term::output;

error ConfigBuildError { NotEnoughArguments }

enum Expr { Num(int) Add(Expr, Expr) }

object Config {
  query: string;
  build(args: string[]) -> ConfigBuildError!Self {
    return Self(args);
  }
}

trait Shape {
  area() -> float;
}

const LIMIT = 10;

test "adds" {
  assert(add(1, 2) == 3);
}

fn add(a: int, b: int) -> int {
  return (a + b);
}
```

- `#[attr]` lines go immediately above the item they decorate.
- No return type → nothing between `)` and `{`. `-> !` is printed as `-> !`.
- Trait methods have no body: `area() -> float;`

### Statements

```
x = 1;
self.count = 0;
items[0] = 2;
while (cond) { ... }
if (cond) { ... } else if (cond2) { ... } else { ... }
match (e) {
  Num(v) => { ... }
  _ => { ... }
}
loop { ... }
break;
continue;
return;
return e;
```

`if`/`while`/`match` conditions are always parenthesized.
`for` and list comprehensions are **gone** after lowering (see below).

### Expressions

| Shape | Printed |
|---|---|
| int / float | `42`, `3.14` |
| string | `"..."` with `\n \t \r \\ \"` escapes |
| f-string | `f"..."` with `{expr-psoma}` |
| bool / null | `true`, `false`, `null` |
| name | `x` |
| self | `self` |
| list | `[1, 2, 3]` |
| call | `f(a, b)`, named: `f(radius = 2.0)` |
| method call | `x.add(y)` |
| field | `x.y` |
| index | `x[y]` |
| unary | `(-x)`, `(not x)` |
| binary | **always parenthesized**: `(a + b)`, `((a + b) * c)` |
| range | `(1..4)`, `(..4)` |
| lambda | `(x) => e`, `(a, b) => e` |
| propagate | `propagate(e)` — should never appear after `desugar::errors` |
| comprehension | `__comprehension((x) => e, iter)` — after lowering |

Binary operators keep their source spelling: `+ - * / % == != < > <= >= and or`.

## Lowering guarantees

`desugar::lower` runs, in order:

1. **append** — `L + x` → `L.add(x)`, `L + M` → `L.extend(M)` (**by the
   checker-selected meaning of `+`; for now: receiver is `.add` unless the
   checker has run**). Documented in `design/02-syntax.md`.
2. **comprehension** — `[e for x in it]` → `__comprehension((x) => e, it)`;
   `for x in it { body }` → cursor `while` with fresh names (`__it0`, `__it1`, …).
3. **errors** — `e!` and `e ! handler` → explicit `match` on the error union.

`psoma::emit` must be a pure function of the AST: same AST → same bytes.
