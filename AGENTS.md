# AimRust — agent instructions

## Project
Aim trainer in Rust. Bevy 0.20.1 + bevy_egui 0.43.1, edition 2024. Binary crate `AimRust` (Cargo.lock tracked).
Single binary. No web, no async runtime beyond Bevy's.

## Commands
- Check: `cargo check`
- Lint: `cargo clippy` (do not introduce new warnings)
- Test: `cargo test`
- Run: `cargo run`

## Layout
- `src/main.rs` — app bootstrap: `CorePlugin`, `PropsPlugin`, `TasksPlugin`, window app ID config.
- `src/core/*` — game core:
  - `state.rs`, `round.rs`, `score.rs`, `storage.rs`: Game states (`AppState`, `TaskState`), timer/countdown, hit/miss tracking, and local score persistence.
  - `config.rs`, `settings.rs`, `paths.rs`: Runtime config (`Settings`), portable mode resolution, and file paths.
  - `hit.rs`, `hit_zone.rs`, `task_api.rs`: Raycasting, hit detection, target interaction contract (`HitMessage`, `MissMessage`).
  - `audio.rs`: Sound generation/playback for Hit and Miss effects with volume control.
  - `discord/`: Zero-dependency Discord Rich Presence IPC client (`DiscordIpcPlugin`).
  - `camera.rs`, `crosshair.rs`, `input.rs`, `ads.rs`, `video.rs`: First-person controller, dynamic crosshairs, ADS, and windowing.
  - `hud.rs`, `arena.rs`: Task runtime HUD and arena spawning wrappers.
  - `ui/*`: All egui screens (`main_menu`, `hud`, `settings`, `results`, `widgets/`).
- `src/world/*` — 3D environments (`default_arena`, `cylinder_arena`, `graybox_arena`, `open_horizon_arena`).
- `src/props/*` — target/gun/environment entity builders (`targets.rs`, `gun.rs`, `environment.rs`).
- `src/tasks/*` — self-contained aim task modules registered via `register_tasks!(...)` in `src/tasks/mod.rs`:
  - Tasks: `sandbox`, `gridshot`, `sixshot`, `spidershot`, `flick`, `micro_adjust`, `tracking_follow`, `tracking_shoot`, `cylinder_micro`, `peek_exercise`, `multishot_ultimate`.
- `packaging/` — Linux packaging (AppImage script, desktop entry `io.github.michaelx17.AimRust`, AppStream metainfo).
- `assets/` — textures, models, and audio. Full task guide in `docs/ADDING_A_TASK.md`.

## Adding a task
1. Copy `src/tasks/_template/mod.rs` → `src/tasks/<name>/mod.rs`.
2. Define `INFO` with metadata, target counts, duration, and instructions.
3. Register `<name>` in `register_tasks!(...)` within `src/tasks/mod.rs`.
4. Ensure targets obey `Settings.colors.target` and follow the standard task interface (no custom in-task egui).
5. Run check + clippy; verify a full round. See `.agents/rules/task-module.md` and `docs/ADDING_A_TASK.md`.

## Egui boundary
All UI (menus/HUD/settings/results) is strictly confined to `src/core/ui/*`; tasks never draw egui directly. See `.agents/rules/egui-boundary.md`.

## Conventions
- Bevy 0.20: messages (`MessageReader`/`MessageWriter`), not events.
- Task-scoped types prefixed with task name; marker components; `#[allow(dead_code)]` on unused helpers.
- No new crates without explicit approval; no core changes outside the contract; no renames outside your target folder; config is read-only for tasks.
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
