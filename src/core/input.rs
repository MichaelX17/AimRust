use bevy::input::mouse::{MouseButton, MouseMotion};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::core::ads::{AdsState, HEADHUNTER_ZOOM_RATIO};
use crate::core::camera::{PlayerCamera, PLAYER_POS};
use crate::core::config::Settings;
use crate::core::state::AppState;

/// Degrees per mouse count at sensitivity 1.0.
pub const BASE_DEG_PER_COUNT: f32 = 0.022;

/// Accumulated look angles in degrees.
#[derive(Resource, Default)]
pub struct PlayerLook {
    pub yaw: f32,
    pub pitch: f32,
}

/// When true the cursor is free and look/fire are disabled.
#[derive(Resource, Default)]
pub struct CursorFree(pub bool);

/// Sent on left click in gameplay (consumed by hit detection).
#[derive(Message)]
pub struct Shot;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerLook>()
            .init_resource::<CursorFree>()
            .add_message::<Shot>()
            .add_systems(OnEnter(AppState::Countdown), reset_look)
            .add_systems(Update, (sync_cursor, update_fov))
            .add_systems(
                Update,
                (mouse_look.after(sync_cursor), shoot).run_if(in_state(AppState::Playing)),
            );
    }
}

fn reset_look(
    mut look: ResMut<PlayerLook>,
    mut cameras: Query<&mut Transform, With<PlayerCamera>>,
) {
    *look = PlayerLook::default();
    if let Some(mut transform) = cameras.iter_mut().next() {
        transform.translation = PLAYER_POS;
        transform.rotation = Quat::IDENTITY;
    }
}

/// Grabs/hides the cursor during Countdown and Playing, frees it elsewhere.
fn sync_cursor(
    state: Res<State<AppState>>,
    mut cursor_free: ResMut<CursorFree>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !state.is_changed() {
        return;
    }

    let in_game = matches!(*state.get(), AppState::Countdown | AppState::Playing);
    cursor_free.0 = !in_game;

    for mut cursor in &mut cursors {
        cursor.visible = !in_game;
        cursor.grab_mode = if in_game {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
    }
}

pub fn mouse_look(
    settings: Res<Settings>,
    cursor_free: Res<CursorFree>,
    ads: Res<AdsState>,
    mut look: ResMut<PlayerLook>,
    mut motion: MessageReader<MouseMotion>,
    mut cameras: Query<&mut Transform, With<PlayerCamera>>,
) {
    if cursor_free.0 {
        motion.clear();
        return;
    }

    let mut delta = Vec2::ZERO;
    for event in motion.read() {
        delta += event.delta;
    }
    if delta == Vec2::ZERO {
        return;
    }

    let zoom = 1.0 + (HEADHUNTER_ZOOM_RATIO - 1.0) * ads.progress;
    let deg = (settings.sens / zoom) * BASE_DEG_PER_COUNT;
    look.yaw -= delta.x * deg;
    look.pitch = (look.pitch - delta.y * deg).clamp(-89.0, 89.0);

    if let Some(mut transform) = cameras.iter_mut().next() {
        transform.translation = PLAYER_POS;
        transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            look.yaw.to_radians(),
            look.pitch.to_radians(),
            0.0,
        );
    }
}

/// Derives the vertical FOV from the horizontal FOV and window aspect.
fn update_fov(
    settings: Res<Settings>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Projection, With<PlayerCamera>>,
    mut applied: Local<Option<(f32, f32)>>,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };
    let aspect = window.width() / window.height().max(1.0);

    if *applied == Some((settings.fov_h, aspect)) {
        return;
    }
    *applied = Some((settings.fov_h, aspect));

    let half_h = settings.fov_h.to_radians() * 0.5;
    let vertical = 2.0 * (half_h.tan() / aspect).atan();

    for mut projection in &mut cameras {
        if let Projection::Perspective(perspective) = &mut *projection {
            perspective.fov = vertical;
        }
    }
}

fn shoot(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    cursor_free: Res<CursorFree>,
    mut ads_state: ResMut<crate::core::settings::HeadhunterAdsState>,
    mut shots: MessageWriter<Shot>,
) {
    if !ads_state.fire_cooldown.is_finished() {
        ads_state.fire_cooldown.tick(time.delta());
    }

    if cursor_free.0 || !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    if ads_state.is_ads {
        // Enforce 0.25s cadence in ADS
        if !ads_state.fire_cooldown.is_finished() {
            return;
        }
        ads_state.fire_cooldown.reset();

        // Recoil pitch +0.042 rad up to cap of 0.14 rad
        ads_state.current_recoil_pitch = (ads_state.current_recoil_pitch + 0.042).min(0.14);

        // Tiny horizontal jitter ±0.005 rad
        let jitter = if ((time.elapsed_secs_wrapped() * 1000.0) as u32).is_multiple_of(2) { 0.005 } else { -0.005 };
        ads_state.current_recoil_yaw = (ads_state.current_recoil_yaw + jitter).clamp(-0.02, 0.02);

        // Reset gun kick timer
        ads_state.gun_kick_timer.reset();
    }

    shots.write(Shot);
}
