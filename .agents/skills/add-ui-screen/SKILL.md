---
name: add-ui-screen
description: Add a new egui screen to the app. Use when adding a new menu, settings page, or overlay.
---

# Add an egui screen

1. Create `src/core/ui/<name>.rs`.
2. Add `pub mod <name>;` to `src/core/ui/mod.rs`.
3. Add the variant to `AppState` in `src/core/state.rs`.
4. Register it in the state-driven UI plugin (search for `menu_ui` in `src/core/ui/mod.rs`).
5. Write the function using the skeleton from `.agents/rules/egui-boundary.md`.
6. Navigate from an existing screen with `next_state.set(AppState::<Name>)`.
7. `cargo check` → fix the first error → repeat.

Keep the screen under 200 lines. If it grows, split private `fn`s in the same file.
