1.  **Define `JsonPrinter`**: Create a stateful JSON builder in `src/emit_json.rs` that maintains indentation and handles comma insertion correctly. It will have `object()` and `array()` methods returning scoped builders for safe formatting.
2.  **Implement AST traversal**: Implement serialization for `Item`, `ItemKind`, `Expr`, `Stmt`, `TypeRepr`, `Pattern`, and all related sub-structs (like `FnDef`, `ObjectDef`, etc.). Map every variant to its name and fields.
3.  **Handle special instructions**:
    -   Unknown/Null kinds: Emit `{"kind": "unknown"}` for any unrecognized variants (handled by `_ =>`).
    -   Properly escape string fields (`"`, `\`, `\n`, `\t`, `< 0x20`).
    -   Output 2-space indented JSON with a trailing newline.
4.  **Create tests in `tests/json_dump.rs`**: Write tests that parse `tests/golden/arith.soma` and ensure its snapshot matches the JSON dump. Create a tiny JSON validator in the test to ensure all outputs are syntactically valid JSON (bracket/quote/escape balance). Also validate all 14 examples.
5.  **Pre-commit steps**: Run required tests and pre-commit checks.
6.  **Submit**: Commit and submit the code.
