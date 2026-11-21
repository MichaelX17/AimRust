---
trigger: glob
globs: "**/*.rs"
---

# Rust + Bevy 0.20 cheat sheet

- **Bevy `=0.20.1`**, `bevy_egui =0.43.1`, edition 2024. Do not use APIs from other versions.
- Systems: prefer `#[derive(SystemParam)]` when a system has >4 params (see `HomeData` in `core/ui/menu.rs`).
- Window: `Query<&Window, With<PrimaryWindow>>`; `single()` returns `Result`.
- Events are **messages**: use the writer type present in `src/` (verify: `MessageWriter` or `EventWriter`).
- State: `ResMut<NextState<AppState>>`; call `.set(AppState::X)`.
- Early exit: `let Ok(x) = query.single() else { return; };` — never `unwrap()`.
- If an API is unknown, read the installed source at `~/.cargo/registry/src/*/bevy-0.20.1/` before guessing. Never invent method names.

Example system skeleton:
```rust
pub fn my_system(mut next: ResMut<NextState<AppState>>, q: Query<&Window, With<PrimaryWindow>>) {
    let Ok(_window) = q.single() else { return; };
    next.set(AppState::Menu);
}
```
