use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::render::view::DebandDither;

use crate::core::ads::AdsState;
use crate::core::input::{PlayerLook, Shot};
use crate::core::settings::{HeadhunterAdsState, UserSettings};

/// Eye position of the stationary player.
pub const PLAYER_POS: Vec3 = Vec3::new(0.0, 1.7, 0.0);

/// Marker for the single persistent player camera.
#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component, Clone, Debug)]
pub struct CameraRecoil {
    pub kick: Vec3,
    pub timer: Timer,
}

impl Default for CameraRecoil {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.2, TimerMode::Once);
        timer.set_elapsed(std::time::Duration::from_secs_f32(0.2));
        Self {
            kick: Vec3::ZERO,
            timer,
        }
    }
}

/// Spawns the one persistent camera at [`PLAYER_POS`], looking down -Z.
pub fn spawn_player_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Tonemapping::None,
        DebandDither::Disabled,
        Transform::from_translation(PLAYER_POS).looking_to(-Vec3::Z, Vec3::Y),
        PlayerCamera,
        CameraRecoil::default(),
    ));
}

/// Recoil system:
/// - In hipfire (No ADS): kicks the camera pitch up on shot and smoothly returns over 0.2s.
/// - In ADS: applies spring-back camera/scope recoil recovery (smoothly decays pitch and yaw, applying kick to view rotation).
pub fn apply_recoil(
    time: Res<Time>,
    settings: Res<UserSettings>,
    ads: Res<AdsState>,
    mut headhunter: ResMut<HeadhunterAdsState>,
    look: Res<PlayerLook>,
    mut shots: MessageReader<Shot>,
    mut camera_query: Query<(&mut Transform, &mut CameraRecoil), With<PlayerCamera>>,
) {
    let Ok((mut transform, mut recoil)) = camera_query.single_mut() else {
        return;
    };

    let mut has_shot = false;
    for _ in shots.read() {
        has_shot = true;
    }

    let in_ads = ads.progress > 0.05 || ads.is_active;
    if has_shot && settings.recoil_enabled && !in_ads {
        recoil.kick = Vec3::new(1.8f32.to_radians(), 0.0, 0.0);
        recoil.timer.reset();
    }

    if in_ads {
        // Cancel hipfire recoil timer
        recoil.timer.set_elapsed(std::time::Duration::from_secs_f32(0.2));
        recoil.kick = Vec3::ZERO;

        // Camera / Scope Recoil Recovery: smoothly decay toward 0.0 with rate 10.0 * dt
        let decay = (10.0 * time.delta_secs()).min(1.0);
        headhunter.current_recoil_pitch *= 1.0 - decay;
        headhunter.current_recoil_yaw *= 1.0 - decay;

        let base_rot = Quat::from_euler(
            EulerRot::YXZ,
            (look.yaw + headhunter.current_recoil_yaw.to_degrees()).to_radians(),
            (look.pitch + headhunter.current_recoil_pitch.to_degrees()).to_radians(),
            0.0,
        );
        transform.rotation = base_rot;
    } else {
        if !recoil.timer.is_finished() {
            recoil.timer.tick(time.delta());
            let factor = 1.0 - recoil.timer.fraction();
            let base_rot = Quat::from_euler(
                EulerRot::YXZ,
                look.yaw.to_radians(),
                look.pitch.to_radians(),
                0.0,
            );
            let recoil_rot = Quat::from_rotation_x(recoil.kick.x * factor);
            transform.rotation = base_rot * recoil_rot;
        }
    }
}
