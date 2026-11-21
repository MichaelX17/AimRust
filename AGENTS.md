# AimRust — agent instructions

## Project
Aim trainer in Rust. Bevy 0.20.1 + bevy_egui 0.43.1, edition 2024. Binary crate (Cargo.lock tracked).
Single binary. No web, no async runtime beyond Bevy's.

## Commands
- Check: `cargo check`
- Lint: `cargo clippy` (do not introduce new warnings)
- Test: `cargo test`
- Run: `cargo run`

## Layout
- `src/main.rs` — app bootstrap: `CorePlugin`, `PropsPlugin`, `TasksPlugin`.
- `src/core/*` — game core (state, hit messages, task API, config, HUD, UI, input, round, score, storage, arena, camera, crosshair, video).
- `src/props/*` — target/gun/environment builders (`spawn_sphere`, `spawn_box`, `TargetMaterial`).
- `src/tasks/<name>/mod.rs` — aim tasks; self-contained.
- `assets/` — assets. `docs/ADDING_A_TASK.md` — full task authoring guide.

## Adding a task
Copy `src/tasks/_template/mod.rs` → `src/tasks/<name>/mod.rs`; set `INFO`; add `<name>` to `register_tasks!(...)` in `src/tasks/mod.rs`; run check + clippy; play a full round. See `.agents/rules/task-module.md` and `docs/ADDING_A_TASK.md`.

## Egui boundary
All UI (menus/HUD/settings/results) is in `src/core/ui/*`; tasks never draw egui. See `.agents/rules/egui-boundary.md`.

## Conventions
- Bevy 0.20: messages (`MessageReader`/`MessageWriter`), not events.
- Task-scoped types prefixed with task name; marker components; `#[allow(dead_code)]` on unused helpers.
- No new crates, no core changes outside the contract, no renames outside your folder, config read-only.
- `cargo check` and `cargo clippy` must be clean.

## Hard rules
1. **Bevy is pinned.** Never suggest APIs from 0.15/0.16/0.18. If unsure, open `Cargo.toml` first.
2. **All UI is egui.** Never add `bevy_ui` nodes, `Text2d`, or `Node` components.
3. **No `unwrap()` / `expect()` in systems.** Bevy 0.20 `single()` returns `Result`; use `let Ok(x) = ... else { return; };`.
4. **Every code edit ends with `cargo check`.** Do not report "done" without a green check.
5. **Do not read or edit `Prompt/`.** It is historical scaffolding, not part of the build.
6. **Prefer small modules.** If a file exceeds ~300 lines, split it.

## Verification loop
Edit → `cargo check` → fix the **first** error only → repeat. Never stack speculative fixes.
