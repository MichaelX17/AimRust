Target Files: src/core/task_api.rs, src/tasks/mod.rs, src/tasks/_template/mod.rs, src/core/mod.rs
Read-Only Files: none
Task_03: Task registry + template

1. task_api.rs:
   - `enum ScoringMode { Click, Tracking }`
   - `struct TaskInfo { id: &'static str, name: &'static str, description: &'static str, scoring: ScoringMode }` (Clone, Copy)
   - `#[derive(Resource, Default)] struct TaskRegistry(pub Vec<TaskInfo>)`
   - `#[derive(Resource, Default)] struct SelectedTask(pub &'static str)`
   - `pub fn task_playing(id: &'static str)` run condition: true when AppState::Playing and SelectedTask == id
   - `pub fn task_selected(id: &'static str)` run condition: SelectedTask == id (any state)
2. tasks/mod.rs: macro `register_tasks!(a, b, ...)` that expands to `mod a; mod b;` and a TasksPlugin which adds each `a::plugin()` and pushes each `a::INFO` into TaskRegistry. `register_tasks!();` (empty) must compile. Call it empty for now.
3. `_template/mod.rs`: `pub const INFO` (id "template", ScoringMode::Click), `pub fn plugin()`, short comments showing: setup OnEnter(Countdown) with task_selected, gameplay systems with task_playing, cleanup OnEnter(Results) and OnEnter(Menu). Declare `#[allow(dead_code)] mod _template;` in tasks/mod.rs (compiled, NOT registered).
4. Register TaskRegistry and SelectedTask in CorePlugin.
5. Done when: check + clippy clean.

Scope: Target Files + Progress.md. Do not read or modify files outside of this scope.