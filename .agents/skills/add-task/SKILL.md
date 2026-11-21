---
name: add-task
description: Scaffold a new aim-training task module. Use when adding a new game mode.
---

# Add a task

1. Copy `src/tasks/_template/` to `src/tasks/<name>/`.
2. Rename the plugin struct to `<Name>Plugin`.
3. Update `TaskInfo`: `id = "<name>"`, `name = "<Name>"`, `description = "..."`, `scoring = ScoringMode::...`.
4. Register in `src/tasks/mod.rs`: add `<name>` to `register_tasks!(..., <name>);`.
5. Insert `TaskInfo` into `TaskRegistry` (handled automatically by `register_tasks!`).
6. `cargo check` → fix the first error → repeat.
7. `cargo run` and verify the task appears on Home and is playable.

Do not touch other tasks. Do not modify `task_api.rs` unless the template requires a new method.
