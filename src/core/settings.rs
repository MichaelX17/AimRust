use std::time::Duration;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Resource, Debug, Clone)]
pub struct HeadhunterAdsState {
    pub is_ads: bool,
    pub fire_cooldown: Timer,
    pub current_recoil_pitch: f32,
    pub current_recoil_yaw: f32,
    pub gun_kick_timer: Timer,
}

impl Default for HeadhunterAdsState {
    fn default() -> Self {
        let mut fire_cooldown = Timer::from_seconds(0.25, TimerMode::Once);
        fire_cooldown.tick(Duration::from_secs(1));
        let mut gun_kick_timer = Timer::from_seconds(0.18, TimerMode::Once);
        gun_kick_timer.tick(Duration::from_secs(1));
        Self {
            is_ads: false,
            fire_cooldown,
            current_recoil_pitch: 0.0,
            current_recoil_yaw: 0.0,
            gun_kick_timer,
        }
    }
}

#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
pub struct GunSettings {
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub color: (f32, f32, f32, f32),
    pub is_translucent: bool,
}

impl Default for GunSettings {
    fn default() -> Self {
        Self {
            scale: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            color: (1.0, 1.0, 1.0, 1.0),
            is_translucent: false,
        }
    }
}

#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
pub struct UserSettings {
    pub gun: GunSettings,
    pub doa_mode_enabled: bool,
    pub recoil_enabled: bool,
    pub sniper_zoom_fov: f32,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            gun: GunSettings::default(),
            doa_mode_enabled: false,
            recoil_enabled: true,
            sniper_zoom_fov: 30.0,
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("AimRust").join("settings.ron"))
}

pub fn load_settings() -> UserSettings {
    let Some(path) = settings_path() else {
        return UserSettings::default();
    };

    if !path.exists() {
        return UserSettings::default();
    }

    let Ok(contents) = fs::read_to_string(&path) else {
        return UserSettings::default();
    };

    ron::from_str(&contents).unwrap_or_default()
}

#[allow(dead_code)]
pub fn save_settings(settings: &UserSettings) {
    let Some(path) = settings_path() else {
        return;
    };

    if let Some(parent) = path.parent()
        && let Err(err) = fs::create_dir_all(parent)
    {
        warn!("Failed to create settings directory {}: {err}", parent.display());
        return;
    }

    let pretty_config = ron::ser::PrettyConfig::default();
    let Ok(serialized) = ron::ser::to_string_pretty(settings, pretty_config) else {
        warn!("Failed to serialize settings to RON");
        return;
    };

    if let Err(err) = fs::write(&path, serialized) {
        warn!("Failed to write settings to {}: {err}", path.display());
    }
}
