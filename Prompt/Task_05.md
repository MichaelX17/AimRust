Target Files: src/core/ui/settings.rs, src/core/video.rs, src/core/ui/mod.rs, src/core/mod.rs
Read-Only Files: none
Task_05: Settings screen + video

1. settings.rs: `pub fn settings_panel(ui: &mut egui::Ui, settings: &mut Settings)` with all controls:
   - sens slider 0.1-10.0
   - fov_h slider 60-130
   - crosshair: style combo, size, thickness, gap, color picker, outline checkbox
   - 5 color pickers (floor, ceiling, walls, target, background_accent)
   - video: vsync, fps_cap (0 = off), window_mode combo
   - show_gun checkbox
   - Reset to defaults button (Settings::default())
2. AppState::Settings screen: egui window using settings_panel + Back button (-> Menu). No crosshair preview.
3. video.rs: system applying video settings on change: vsync (present mode), window mode, fps cap (simple sleep-based limiter, 0 = off, no extra crates).
4. Done when: check + clippy clean; `cargo run`: changes persist after restart; Reset works; vsync/window mode apply live.

Scope: Target Files + Progress.md. Do not read or modify files outside of this scope.