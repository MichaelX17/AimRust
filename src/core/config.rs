use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::core::paths;

const CONFIG_FILE: &str = "settings.toml";
const SAVE_DEBOUNCE_SECS: f32 = 0.5;

#[derive(Resource, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub sens: f32,
    pub fov_h: f32,
    pub crosshair: CrosshairCfg,
    pub colors: ColorsCfg,
    pub video: VideoCfg,
    pub audio: AudioCfg,
    pub show_gun: bool,
    pub hold_to_ads: bool,
    pub recoil_enabled: bool,
    pub camera_shake_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sens: 1.0,
            fov_h: 103.0,
            crosshair: CrosshairCfg::default(),
            colors: ColorsCfg::default(),
            video: VideoCfg::default(),
            audio: AudioCfg::default(),
            show_gun: true,
            hold_to_ads: true,
            recoil_enabled: true,
            camera_shake_enabled: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CrosshairCfg {
    pub style: CrosshairStyle,
    pub size: f32,
    pub thickness: f32,
    pub gap: f32,
    pub color: [u8; 3],
    pub outline: bool,
}

impl Default for CrosshairCfg {
    fn default() -> Self {
        Self {
            style: CrosshairStyle::Dot,
            size: 10.0,
            thickness: 2.0,
            gap: 4.0,
            color: [0, 255, 102],
            outline: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrosshairStyle {
    #[default]
    Dot,
    Cross,
    Circle,
    T,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorsCfg {
    pub floor: [u8; 3],
    pub ceiling: [u8; 3],
    pub walls: [u8; 3],
    pub target: [u8; 3],
    pub background_accent: [u8; 3],
}

impl Default for ColorsCfg {
    fn default() -> Self {
        Self {
            floor: [43, 45, 49],
            ceiling: [28, 29, 33],
            walls: [58, 61, 68],
            target: [255, 90, 54],
            background_accent: [79, 140, 255],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VideoCfg {
    pub vsync: bool,
    pub fps_cap: u32,
    pub window_mode: WindowMode,
}

impl Default for VideoCfg {
    fn default() -> Self {
        Self {
            vsync: true,
            fps_cap: 0,
            window_mode: WindowMode::Windowed,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowMode {
    #[default]
    Windowed,
    Borderless,
    Fullscreen,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioCfg {
    pub enabled: bool,
    /// Master SFX volume percentage (`0..=100`).
    pub volume: u32,
}

impl Default for AudioCfg {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 80,
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    paths::config_file_path(CONFIG_FILE)
}

/// Loads settings at startup: missing file -> defaults, unparsable file -> `.bak`
/// rename -> defaults. Always inserts the `Settings` resource.
pub fn load_settings(mut commands: Commands) {
    commands.insert_resource(load_or_default());
}

fn load_or_default() -> Settings {
    let Some(path) = settings_path() else {
        return Settings::default();
    };

    let settings = match std::fs::read_to_string(&path) {
        Ok(text) => match toml::from_str::<Settings>(&text) {
            Ok(settings) => settings,
            Err(_) => {
                let _ = std::fs::rename(&path, backup_path(&path));
                Settings::default()
            }
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Settings::default(),
        Err(_) => {
            let _ = std::fs::rename(&path, backup_path(&path));
            Settings::default()
        }
    };

    write_settings(&path, &settings);
    settings
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("toml.bak")
}

fn write_settings(path: &Path, settings: &Settings) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = toml::to_string_pretty(settings) {
        let _ = std::fs::write(path, text);
    }
}

/// Writes the current settings to `settings.toml`.
pub fn save_settings(settings: &Settings) {
    if let Some(path) = settings_path() {
        write_settings(&path, settings);
    }
}

/// Saves 0.5 s after the `Settings` resource last changed.
pub fn autosave_settings(
    settings: Res<Settings>,
    time: Res<Time>,
    mut pending: Local<Option<Timer>>,
) {
    if settings.is_changed() {
        *pending = Some(Timer::from_seconds(SAVE_DEBOUNCE_SECS, TimerMode::Once));
    }
    let Some(timer) = pending.as_mut() else {
        return;
    };
    timer.tick(time.delta());
    if timer.just_finished() {
        save_settings(&settings);
        *pending = None;
    }
}
