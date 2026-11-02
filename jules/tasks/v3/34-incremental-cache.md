# Task 34 — Incremental parse/check cache

Read `jules/README.md`, `design/08-pipeline.md` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `src/cache.rs` (new file)
- `tests/cache.rs` (new file)

## Goal
Avoid re-parsing unchanged files in directory mode:
- `cache::Cache<P: AsCacheDir>` with `get_or_parse(path, source) -> Arc<[Item]>`;
- key = (path, hash of source) using a hand-rolled FNV-1a 64; value = parsed items
  serialised with `emit_json` (reuse it, do not reimplement);
- `Cache::stats() -> CacheStats { hits, misses }`; `Cache::clear()`;
- a cache hit returns items structurally identical to a fresh parse (test with
  `PartialEq`);
- corrupt cache entries are ignored and re-parsed, never a panic.

## Acceptance
- `tests/cache.rs`: second parse of the same source is a hit; changed source is a
  miss; corrupt file → miss; stats correct; identical items after hit.
- In-memory backend for tests plus a directory backend behind a trait, so tests
  never touch the real filesystem outside `tests/tmp/`.

## Hard rules
Only the two files above. No new deps. `cargo test` must pass. No `todo!()`.
