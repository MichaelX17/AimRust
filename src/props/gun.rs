use std::path::Path;

use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::core::camera::PlayerCamera;
use crate::core::config::Settings;
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
struct GunViewmodel;

pub struct GunPlugin;

impl Plugin for GunPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Countdown), spawn_gun)
            .add_systems(Update, sync_gun)
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
        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(GUN_SCENE))),
        Transform::from_translation(GUN_OFFSET)
            .with_rotation(GUN_ROT)
            .with_scale(Vec3::splat(GUN_SCALE)),
        ChildOf(camera),
        GunViewmodel,
    ));
    true
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
