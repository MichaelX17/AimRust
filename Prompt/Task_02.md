Target Files: src/core/config.rs, src/core/mod.rs
Read-Only Files: none
Task_02: Settings struct + settings.toml

1. config.rs: `Settings` (Resource, Clone, Serialize, Deserialize, `#[serde(default)]`) with EXACT fields:
   - sens: f32 (default 1.0)
   - fov_h: f32 (103.0)
   - crosshair: CrosshairCfg { style: CrosshairStyle{Dot,Cross,Circle,T}, size: f32 10.0, thickness: f32 2.0, gap: f32 4.0, color: [u8;3] [0,255,102], outline: bool true }
   - colors: ColorsCfg { floor [43,45,49], ceiling [28,29,33], walls [58,61,68], target [255,90,54], background_accent [79,140,255] } (all [u8;3])
   - video: VideoCfg { vsync: bool true, fps_cap: u32 0, window_mode: WindowMode{Windowed,Borderless,Fullscreen} }
   - show_gun: bool true
2. Path: dirs::config_dir()/aim_trainer/settings.toml (create dir if missing).
3. Startup load: missing -> defaults + write; parse error -> rename file to settings.toml.bak, defaults + write. No unwrap on I/O or parsing.
4. `pub fn save_settings(&Settings)`. System: when the Settings resource changes, save (debounce 0.5 s).
5. Register in CorePlugin.
6. Done when: check + clippy clean; `cargo run` creates settings.toml with defaults.

Scope: Target Files + Progress.md. Do not read or modify files outside of this scope.