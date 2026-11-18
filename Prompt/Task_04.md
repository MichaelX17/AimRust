Target Files: src/core/ui/mod.rs, src/core/ui/menu.rs, src/core/mod.rs
Read-Only Files: none (Settings fields are in Progress.md Interfaces)
Task_04: Main menu + theme

1. Add bevy_egui plugin. ui/mod.rs: UiPlugin, registered in CorePlugin.
2. menu.rs: in AppState::Menu show a centered egui menu: title "AimTrainer", buttons Play (-> TaskSelect), Settings (-> Settings), Quit (send AppExit).
3. Theme system: set ClearColor and egui accent (selection/hover/buttons) from Settings.colors.background_accent; re-apply when Settings changes. Keep text readable (light text on dark UI).
4. Keyboard + mouse friendly. Idle CPU must stay low (no per-frame heavy work).
5. Done when: check + clippy clean; `cargo run` shows the menu; Quit works; Play/Settings change state (screens come later).

Scope: Target Files + Progress.md. Do not read or modify files outside of this scope.