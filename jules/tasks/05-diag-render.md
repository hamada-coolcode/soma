# Task 05 — diag: ariadne rendering

Read `design/02-syntax.md` (error style lives in the docs' examples) and
`src/diag/{error.rs,render.rs,sourcemap.rs,span.rs}`.

## Files you may touch (ONLY these)
- `src/diag/render.rs`
- `src/diag/error.rs`

## Goal
Implement `render(diag: &Diagnostic, map: &SourceMap) -> String` using
`ariadne` 0.6 (`ariadne::Report`, `ReportKind::Error`, `.with_label(...)`,
`.with_help(...)`, `finish().write(&mut buf, source)`).
- Label text = the message; label span = the diagnostic span.
- Include the file name and 1-based `line:col`.
- If `diag.help` is set, attach it as a help note.
- Keep `ErrorKind`/`Diagnostic` API; you may add `Display` impls.

Output must be non-empty and deterministic.

## Tests
Build a `SourceMap` with a small file, a `Diagnostic` at a known offset, assert
the rendered text contains: the message, the file name, `line:col`, and (when
set) the help text. One test per `ErrorKind` mapping if you add labels.

## Hard rules
Only edit the two files above. Never edit `Cargo.toml`, any `mod.rs`,
`design/`, `examples/`, `jules/`. `ariadne` is already a dependency.
