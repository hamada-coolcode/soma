# Task 36 — Multi-file project fixture + end-to-end test

Read `jules/README.md`, `design/08-pipeline.md` and `jules/psoma.md` first.

## Files you may touch (ONLY these)
- `tests/project/multi/*.soma` (new files)
- `tests/project.rs` (new file)

## Goal
Prove the loader + driver work on a real multi-file project:
- `tests/project/multi/main.soma` imports `app.util` and `app.model`;
- `app/util.soma` defines helper functions, `app/model.soma` defines an object;
- the project must parse, pass `loader::load_entry`, and check cleanly;
- add a deliberate broken variant (`main_broken.soma`) importing a missing module
  and assert the `E0701` diagnostic names the searched paths.

## Acceptance
- `tests/project.rs`: clean project loads all 3 files in order, no diagnostics;
  broken project yields exactly one `E0701`; directory check over the fixture
  returns exit-code-equivalent failure.
- No changes to `src/` at all — this is a test-only task.

## Hard rules
Only the files above. No new deps. `cargo test` must pass.
