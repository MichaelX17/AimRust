Target Files: Cargo.toml, .gitignore, src/main.rs, src/core/mod.rs, src/core/state.rs, src/core/camera.rs, src/props/mod.rs, src/tasks/mod.rs
Read-Only Files: none
Task_01: Scaffold, states, camera

1. Run `rustc --version` and `cargo --version`. If either is missing: STOP, tell user to install rustup.
2. If no Cargo.toml: `cargo init --name aim_trainer`.
3. Dependencies: bevy (latest stable, exact-pinned with `=`, default-features=false, enable only: window/winit, 3D rendering + PBR, glTF loading, default font, input, diagnostics), bevy_egui (compatible version), serde (derive), toml, dirs. Record exact versions in Progress.md Meta.
4. Profiles: dev opt-level=1, dev.package."*" opt-level=3; release opt-level=3, lto="thin", codegen-units=1, strip=true, panic="abort".
5. .gitignore: /target
6. state.rs: `AppState` enum (Menu, Settings, TaskSelect, TimeSelect, Countdown, Playing, Paused, Results), derive States, Default = Menu.
7. camera.rs: `pub const PLAYER_POS: Vec3 = Vec3::new(0.0, 1.7, 0.0)`; `PlayerCamera` marker component; Startup system spawning ONE persistent Camera3d at PLAYER_POS looking toward -Z.
8. Empty plugins: CorePlugin (core/mod.rs; registers state + camera), PropsPlugin (props/mod.rs), TasksPlugin (tasks/mod.rs).
9. main.rs: DefaultPlugins (title "AimTrainer", 1280x720), add the 3 plugins.
10. Done when: check + clippy clean; `cargo run` opens a window and closes cleanly.

Scope: Target Files + Progress.md. Do not read or modify files outside of this scope.