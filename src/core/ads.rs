use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::core::camera::PlayerCamera;
use crate::core::config::Settings;
use crate::core::crosshair::CrosshairRoot;
use crate::core::input::CursorFree;
use crate::core::state::AppState;
use crate::props::gun::{GUN_OFFSET, GUN_ROT, Gun};

/// Valorant Chamber Headhunter ADS zoom ratio (exact 1.5x zoom).
pub const HEADHUNTER_ZOOM_RATIO: f32 = 1.50;

/// Viewmodel placement in ADS (centered horizontally, kept at bottom of screen below the optic).
pub const HEADHUNTER_ADS_GUN_OFFSET: Vec3 = Vec3::new(0.0, -0.27, -0.42);

/// Speed of ADS animation transition (18.0 = snappy ~80ms transition).
const ADS_TRANSITION_SPEED: f32 = 18.0;

#[derive(Resource, Debug, Clone)]
pub struct AdsState {
    /// 0.0 = hipfire, 1.0 = full ADS
    pub progress: f32,
    /// Whether right-click is actively held in gameplay
    pub is_active: bool,
}

impl Default for AdsState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            is_active: false,
        }
    }
}

pub struct AdsPlugin;

impl Plugin for AdsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AdsState>()
            .add_systems(
                Update,
                (
                    update_ads_input,
                    apply_ads_camera,
                    apply_ads_gun,
                    apply_ads_crosshair,
                )
                    .chain(),
            )
            .add_systems(OnEnter(AppState::Menu), reset_ads)
            .add_systems(OnEnter(AppState::Paused), reset_ads)
            .add_systems(OnEnter(AppState::Results), reset_ads)
            .add_systems(OnEnter(AppState::TaskSelect), reset_ads)
            .add_systems(OnEnter(AppState::TimeSelect), reset_ads);
    }
}

/// Detects right-click in gameplay and smoothly updates ADS progress.
fn update_ads_input(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    cursor_free: Res<CursorFree>,
    state: Res<State<AppState>>,
    settings: Option<Res<Settings>>,
    mut ads: ResMut<AdsState>,
    mut headhunter: ResMut<crate::core::settings::HeadhunterAdsState>,
) {
    let in_game = matches!(*state.get(), AppState::Playing | AppState::Countdown);
    let hold_to_ads = settings.as_deref().map(|s| s.hold_to_ads).unwrap_or(true);

    if !in_game || cursor_free.0 {
        ads.is_active = false;
    } else if hold_to_ads {
        ads.is_active = buttons.pressed(MouseButton::Right);
    } else if buttons.just_pressed(MouseButton::Right) {
        ads.is_active = !ads.is_active;
    }

    let target = if ads.is_active { 1.0 } else { 0.0 };
    let step = (time.delta_secs() * ADS_TRANSITION_SPEED).min(1.0);
    ads.progress = ads.progress + (target - ads.progress) * step;

    if (ads.progress - target).abs() < 0.001 {
        ads.progress = target;
    }

    headhunter.is_ads = ads.progress > 0.5;
}

/// Applies a slight zoom (1.15x) to the camera when aiming down sights.
fn apply_ads_camera(
    ads: Res<AdsState>,
    settings: Option<Res<Settings>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Projection, With<PlayerCamera>>,
    mut was_active: Local<bool>,
) {
    if ads.progress <= 0.0 && !*was_active {
        return;
    }
    let Some(settings) = settings else {
        return;
    };
    *was_active = ads.progress > 0.0;

    let Ok(window) = windows.single() else {
        return;
    };
    let aspect = window.width() / window.height().max(1.0);

    let zoom = 1.0 + (HEADHUNTER_ZOOM_RATIO - 1.0) * ads.progress;
    let current_fov_h = settings.fov_h / zoom;
    let half_h = (current_fov_h * 0.5).to_radians();
    let vertical = 2.0 * (half_h.tan() / aspect).atan();

    for mut projection in &mut cameras {
        if let Projection::Perspective(perspective) = &mut *projection {
            perspective.fov = vertical;
        }
    }
}

/// Moves the gun viewmodel to the center of the screen when aiming down sights,
/// and applies tight ADS gun kick animation when shooting in ADS.
fn apply_ads_gun(
    time: Res<Time>,
    ads: Res<AdsState>,
    settings: Option<Res<Settings>>,
    mut headhunter: ResMut<crate::core::settings::HeadhunterAdsState>,
    mut guns: Query<&mut Transform, With<Gun>>,
    mut was_active: Local<bool>,
) {
    if ads.progress <= 0.0 && !*was_active {
        return;
    }
    *was_active = ads.progress > 0.0;

    // Tick gun kick timer in ADS
    if !headhunter.gun_kick_timer.is_finished() {
        headhunter.gun_kick_timer.tick(time.delta());
    }

    let recoil_enabled = settings.is_none_or(|s| s.recoil_enabled);

    let k = if recoil_enabled && headhunter.is_ads && !headhunter.gun_kick_timer.is_finished() {
        (1.0 - headhunter.gun_kick_timer.fraction()).powi(2)
    } else {
        0.0
    };

    for mut transform in &mut guns {
        if ads.progress > 0.0 {
            let base_pos = GUN_OFFSET.lerp(HEADHUNTER_ADS_GUN_OFFSET, ads.progress);
            // k: kicks slightly back (0.045*k) and dips 8mm downward (-0.008*k)
            transform.translation = base_pos + Vec3::new(0.0, -0.008 * k, 0.045 * k);
            transform.rotation = GUN_ROT * Quat::from_rotation_x(0.075 * k);
        }
    }
}

/// Hides the default hipfire crosshair during ADS so the Chamber optic reticle takes over.
fn apply_ads_crosshair(
    ads: Res<AdsState>,
    mut crosshairs: Query<&mut Visibility, With<CrosshairRoot>>,
    mut was_hidden: Local<bool>,
) {
    let should_hide = ads.progress > 0.4;
    if should_hide == *was_hidden {
        return;
    }
    *was_hidden = should_hide;

    for mut vis in &mut crosshairs {
        *vis = if should_hide {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

/// Resets ADS and restores camera, gun, and crosshair when exiting gameplay.
fn reset_ads(
    mut ads: ResMut<AdsState>,
    mut headhunter: ResMut<crate::core::settings::HeadhunterAdsState>,
    settings: Option<Res<Settings>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Projection, With<PlayerCamera>>,
    mut guns: Query<&mut Transform, With<Gun>>,
    mut crosshairs: Query<&mut Visibility, With<CrosshairRoot>>,
) {
    ads.progress = 0.0;
    ads.is_active = false;
    headhunter.is_ads = false;
    headhunter.current_recoil_pitch = 0.0;
    headhunter.current_recoil_yaw = 0.0;

    for mut vis in &mut crosshairs {
        *vis = Visibility::Inherited;
    }
    for mut transform in &mut guns {
        transform.translation = GUN_OFFSET;
        transform.rotation = GUN_ROT;
    }
    let fov_h = settings.as_deref().map(|s| s.fov_h).unwrap_or(103.0);
    if let Ok(window) = windows.single() {
        let aspect = window.width() / window.height().max(1.0);
        let half_h = (fov_h * 0.5).to_radians();
        let vertical = 2.0 * (half_h.tan() / aspect).atan();
        for mut projection in &mut cameras {
            if let Projection::Perspective(perspective) = &mut *projection {
                perspective.fov = vertical;
            }
        }
    }
}
