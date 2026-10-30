# 02 — Syntax

soma's surface is **indentation-based**, Python-like, with no braces or
semicolons. The design goal: read like an outline, mean like a systems language.

## Indentation

- Indent unit is **custom but must be consistent within a file**: 2, 4, 6 or 8
  spaces all work. **2 is the preferred / idiomatic width.**
- A block starts by indenting under its header and ends by dedenting.
- Comments start with `#` and run to end of line.
- Tabs are not used for indentation.

## Blocks are keyword-led

One keyword introduces a block that can contain several named things at the same
indent level. This applies uniformly:

```
fn
  add a: int b: int -> int
    return a + b

  main
    output(add(2 3))

object
  Circle
    radius: float

  Rectangle
    width: float
    height: float

import
  lib:std
    io
      term
        output

error
  MathError
    DivisionByZero

trait
  Shape
    area -> float
    name -> string
```

### Keywords

| Keyword | Introduces |
| --- | --- |
| `fn` | function definitions |
| `object` | object definitions (was `struct`; renamed to `object`) |
| `trait` | trait (interface) definitions |
| `error` | error type definitions |
| `enum` | enumerations — **TBD**, being rethought (see 04) |
| `import` | module imports (mirrors the module path visually) |
| `const` | named constants |
| `test` | test blocks |
| `export` | exported (public) definitions |
| `if` / `elif` / `else` | conditional |
| `while`, `for … in`, `loop` | iteration (`loop` is infinite; exit with `break`) |
| `break`, `continue`, `return` | control transfer |
| `match` | pattern matching — **TBD** with enums |

## Function signatures

```
name [generics] params -> returnType
```

- **Generics** go in `[]`: declared as `Stack[T]`, applied at the call as
  `Stack.new[int]()`, `input[uint]("...")`, `v.parse[int]()!`.
- **Parameters** are bare `name: type` pairs — **no parens, no commas**:
  `search query: string contents: string`.
- **Return type** follows `->`: `add a: int b: int -> int`. No return type means
  no `->` at all.
- **`!` is a return type**: `main -> !`, `run config: Config -> !` — the
  function *can fail* and returns no success value (see
  [05-errors.md](05-errors.md)).
- **Calls always use parens**, space-separated, no commas: `add(a b)`,
  `s.pop()`, `c.area()`.

Examples:
```
add a: int b: int -> int
div a: int b: int -> MathError!int
build args: string[] -> ConfigBuildError!Self
main -> !
hello -> string
apply x: int f: (int) int -> int
print
```

## Calls — no commas, no `::`

- Arguments are space-separated inside parens: `f(a b)`, `add(a b)`,
  `router.get("/" hello)`.
- Method access and namespacing both use `.` — there is **no `::`**:
  `Router.new("localhost" 7777)`, `Config.build(args)`, `x.lower()`.

## Types

| Form | Meaning |
| --- | --- |
| `int`, `float`, `bool`, `string` | scalars |
| `string[]`, `T[]` | growable list |
| `int[10]`, `T[n]` | fixed-size array (`n` known at compile time) |
| `T?` | optional (absent value is `null`) |
| `(int) int` | function type |
| `Stack[T]` | generic instantiation |

## Strings

- Literal: `"hello"`
- Interpolated: `f"{a} + {b} = {a + b}"` (the `f` prefix, `{…}` embeds an expression)
- Multiline: `""" … """`
- Common methods: `.len() .lines() .contains() .lower() .upper()`

## Operators

| Category | Operators |
| --- | --- |
| Arithmetic | `+  -  *  /  %` |
| Comparison | `==  !=  <  >  <=  >=` |
| Logical | `and  or  not` (TBD: confirm spellings) |
| Range | `a..b` (e.g. `1..100`); **`..b` is the default** — starts at 1, so `..4` = `1..4`. Write `0..b` only when you need zero. |
| List/string | `a + b` — append/extend/concat in place (see below) |
| Error | `!` (propagate), `! handler` (catch) — see 05 |
| Lambda | `x => expr`, `(a b) => expr` |

There is **no reference operator** — references are implicit; `&`, `&mut` and a
reference `*` do not exist (see [03-memory-model.md](03-memory-model.md)).
Arithmetic `*` is plain multiplication.

## List `+` — append and extend

On a list or string, `+` **appends in place** and evaluates to the same value.
It is sugar for `.add()` / `.extend()`:

```
seq = []
seq + 5              # append one element
seq + [6 7]          # extend with a list
names + "hamada"     # strings concat in place too
```

- Numeric `+` keeps its arithmetic meaning; `+` falls back to `.add()` only when
  the left operand isn't a number (so `5 + nums` is just a type error).
- Because it mutates, `x = a + b` **aliases `a`** — it is not Python's copying
  `+`. For a fresh list use `.concat()` or a comprehension.
- Any object with an `add` method gets `+` for free.
- Zero cost: append is a bump-pointer advance on the data stack
  (see [07-collections.md](07-collections.md)).

## Lambdas / closures

```
scale = x => x * factor              # single parameter
sort((a b) => b - a)                 # multiple parameters
numbers.map(x => x * 2)
apply(5 x => x * 2)                  # passed as an argument
```

Closures **capture by reference**, managed by the same scope rules as every
other value — no marker needed.

## Single-line bodies

`if … =>` runs a one-liner on the same line:
```
if guess > SECRET_NUMBER => output("Too big!")
elif guess < SECRET_NUMBER => output("Too small!")
else
  output("You win!")
  break
```

## List comprehensions

```
handles = [worker(i) for i in 0..4]
squares = [x * x for x in nums]
```

`[expr for var in iter]` builds a list. When `expr` calls an `#[async]`
function (see [06-concurrency.md](06-concurrency.md)) it collects handles.

## Attributes

`#[name]` on the line above a definition attaches an attribute:
```
#[async]
worker id: int -> int
```

## Named constructor arguments

Object construction takes `field = value` pairs, space-separated, no commas:
```
Node(val = 1 next = null)
Circle(radius = 2.0)
Stack(items = [])
```
Plain function calls use positional arguments, also without commas: `add(a b)`.

## Test blocks

A `test` block holds named tests. Each test is a `"name"` followed by an
indented body; **the last expression is the assertion** (usually
`expected == actual`):

```
test
  "One Result"
    query = "duct"
    contents = "Duct tape."
    ["safe, fast, productive."] == search(query contents)

  "Case Sensitive"
    query = "Duct"
    ...
```

An empty test body is a TODO.

## Entry point

`main` is the program entry point; `main -> !` if it can fail.
