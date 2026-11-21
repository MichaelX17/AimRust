use std::path::Path;
use std::time::Duration;

use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::core::ads::AdsState;
use crate::core::camera::PlayerCamera;
use crate::core::config::Settings;
use crate::core::input::Shot;
use crate::core::settings::UserSettings;
use crate::core::state::AppState;

/// Viewmodel placement in camera space (right, down, forward).
pub const GUN_OFFSET: Vec3 = Vec3::new(0.20, -0.22, -0.45);
pub const GUN_SCALE: f32 = 0.25;
/// +90 degrees around Y: model barrel (+X) points toward -Z.
pub const GUN_ROT: Quat = Quat::from_xyzw(
    0.0,
    std::f32::consts::FRAC_1_SQRT_2,
    0.0,
    std::f32::consts::FRAC_1_SQRT_2,
);

const GUN_SCENE: &str = "models/pistol.glb";
const GUN_FILE: &str = "assets/models/pistol.glb";

#[derive(Component)]
pub struct Gun;

#[derive(Component)]
struct GunViewmodel;

#[derive(Component)]
pub struct GunShootAnim {
    pub timer: Timer,
    pub kick_z: f32,
    pub pitch_rad: f32,
}

impl Default for GunShootAnim {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.12, TimerMode::Once);
        timer.tick(Duration::from_secs(1));
        Self {
            timer,
            kick_z: 0.0,
            pitch_rad: 0.0,
        }
    }
}

pub struct GunPlugin;

impl Plugin for GunPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Countdown), spawn_gun)
            .add_systems(
                Update,
                (
                    sync_gun,
                    trigger_hipfire_gun_anim,
                    apply_gun_settings_and_anim,
                )
                    .chain(),
            )
            .add_systems(OnEnter(AppState::Results), despawn_gun)
            .add_systems(OnEnter(AppState::Menu), despawn_gun);
    }
}

fn spawn_model(commands: &mut Commands, asset_server: &AssetServer, camera: Entity) -> bool {
    if !Path::new(GUN_FILE).exists() {
        warn!("Pistol viewmodel missing at {GUN_FILE}; skipping gun");
        return false;
    }

    commands.spawn((
        Gun,
        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(GUN_SCENE))),
        Transform::from_translation(GUN_OFFSET)
            .with_rotation(GUN_ROT)
            .with_scale(Vec3::splat(GUN_SCALE)),
        ChildOf(camera),
        GunViewmodel,
        GunShootAnim::default(),
    ));
    true
}

/// Triggers procedural shooting animation on the gun model only when in hip-fire mode (No ADS).
fn trigger_hipfire_gun_anim(
    ads: Res<AdsState>,
    mut shots: MessageReader<Shot>,
    mut gun_query: Query<&mut GunShootAnim, With<Gun>>,
) {
    let mut has_shot = false;
    for _ in shots.read() {
        has_shot = true;
    }

    if !has_shot {
        return;
    }

    // If currently in ADS / zoomed mode, do nothing (keep gun steady)
    let in_ads = ads.progress > 0.05 || ads.is_active;
    if in_ads {
        return;
    }

    for mut anim in &mut gun_query {
        anim.timer.reset();
        anim.kick_z = 0.08;
        anim.pitch_rad = 0.06;
    }
}

/// Updates gun transform with base UserSettings offset and recoil decay animation.
fn apply_gun_settings_and_anim(
    time: Res<Time>,
    ads: Res<AdsState>,
    user_settings: Res<UserSettings>,
    mut gun_query: Query<(&mut Transform, &mut GunShootAnim), With<Gun>>,
) {
    // Only apply hip-fire animation when not actively in ADS
    if ads.progress > 0.05 || ads.is_active {
        return;
    }

    let base_offset = GUN_OFFSET
        + Vec3::new(
            user_settings.gun.offset_x,
            user_settings.gun.offset_y,
            0.0,
        );

    for (mut transform, mut anim) in &mut gun_query {
        if !anim.timer.is_finished() {
            anim.timer.tick(time.delta());
        }
        let t = 1.0 - anim.timer.fraction();

        // Translation: base offset + kick backward toward camera (+Z in camera space)
        transform.translation = base_offset + Vec3::new(0.0, 0.0, anim.kick_z * t);
        // Rotation: base rotation * slight muzzle rise around X
        transform.rotation = GUN_ROT * Quat::from_rotation_x(anim.pitch_rad * t);
    }
}

fn spawn_gun(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<Settings>,
    cameras: Query<Entity, With<PlayerCamera>>,
) {
    if !settings.show_gun {
        return;
    }
    if let Ok(camera) = cameras.single() {
        spawn_model(&mut commands, &asset_server, camera);
    }
}

fn despawn_gun(mut commands: Commands, gun: Query<Entity, With<GunViewmodel>>) {
    for entity in &gun {
        commands.entity(entity).despawn();
    }
}

/// Applies live `show_gun` changes and keeps the gun during pause.
fn sync_gun(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<Settings>,
    state: Res<State<AppState>>,
    cameras: Query<Entity, With<PlayerCamera>>,
    gun: Query<Entity, With<GunViewmodel>>,
) {
    let desired = settings.show_gun
        && matches!(
            *state.get(),
            AppState::Countdown | AppState::Playing | AppState::Paused
        );
    let existing = !gun.is_empty();

    if desired && !existing {
        if let Ok(camera) = cameras.single() {
            spawn_model(&mut commands, &asset_server, camera);
        }
    } else if !desired && existing {
        for entity in &gun {
            commands.entity(entity).despawn();
        }
    }
}
