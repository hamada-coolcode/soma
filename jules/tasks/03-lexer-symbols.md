# Task 03 — lexer symbols: operators + keywords

Read `jules/grammar.ebnf` (§lexical, §operators) and `src/lexer/token.rs`.

## Files you may touch (ONLY these)
- `src/lexer/symbol.rs`

## Goal
Implement `scan_op(src, start) -> Option<(Op, usize)>` with **maximal munch**:

```
#[  ->  =>  ==  !=  <=  >=  ..  ->(arrow)  .
+  -  *  /  %  =  <  >  !  :  ?  (  )  [  ]
```
Order matters: check 2-char forms before 1-char (`#[` before `#`, `->`/`=>`
before `-`/`=`, `==` before `=`, `<=` before `<`, `..` before `.`).
`#` alone is NOT an operator (it starts a comment — the scanner handles it).
Return `None` when no operator matches.

Keep `KEYWORDS`, `is_keyword`, `op_token` as declared.

## Tests
Table-driven: every operator with expected `(Op, end_offset)`; the maximal-munch
cases (`->` vs `-`, `..` vs `.`, `==` vs `=`, `#[` vs `#`); unknown char → `None`;
`is_keyword("fn") == true`, `is_keyword("fnx") == false`.

## Hard rules
Only edit `src/lexer/symbol.rs`. Never edit `Cargo.toml`, any `mod.rs`,
`design/`, `examples/`, `jules/`. No new deps.
