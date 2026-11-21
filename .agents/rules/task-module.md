---
trigger: glob
globs: "src/tasks/**/*.rs"
---

# Task module contract

- Each task is a folder `src/tasks/<name>/mod.rs`. Never add a task as a single `.rs` file.
- Copy `src/tasks/_template/mod.rs` as the starting point.
- Every task exposes a `Plugin` and registers its `TaskInfo` in `TaskRegistry` (see `src/core/task_api.rs`).
- `TaskInfo` fields (verify against facts sheet): `id`, `name`, `description`, `scoring`.
- Do not modify `task_api.rs` unless adding a trait method; if you do, update **every** task and `_template`.
- Do not import from another task module; shared logic goes in `src/core/`.
- After adding a task, run `cargo check` and `cargo run`; the task must appear on Home and be playable.
