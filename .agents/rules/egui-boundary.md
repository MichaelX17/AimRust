---
trigger: glob
globs: "src/core/ui/**/*.rs"
---

# egui UI conventions

- One screen = one `pub fn <name>_ui(mut contexts: EguiContexts, ...)`.
- Always start with:
  ```rust
  let Ok(ctx) = contexts.ctx_mut() else { return; };
  let Ok(window) = windows.single() else { return; };
  let size = crate::core::ui::page_size(ctx, window);
  ```
- Wrap content in `widgets::page(ctx, "<id>", size, |ui| { ... })`.
- Use only existing widgets (verify against facts sheet): `page_inner`, `fixed_card`, `tile`, `primary_button`, `ghost_button`, `chip`, `big_value`, `small_label`, `muted`, `bar_chart`, `thousands`. Do not call raw `egui::Button` / `egui::Frame` or define new colors.
- Colors only from `widgets::{TEXT, TEXT_WEAK, POSITIVE, NEGATIVE, SURFACE_2, BORDER}`; accent comes from `UiAccent`.
- Layout: for the main page grid use exact rects (`allocate_exact_size` + `new_child` + `UiBuilder`), as in `menu.rs`. Never use `horizontal_top` for the main grid.
- Navigation: `next_state.set(AppState::X)`; never push/pop screens manually.
- Data: `Res<TaskRegistry>`, `ResMut<SelectedTask>`, `Res<Scores>`; never query the ECS directly from UI.
