use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::render::view::DebandDither;
use bevy::prelude::*;

/// Eye position of the stationary player.
pub const PLAYER_POS: Vec3 = Vec3::new(0.0, 1.7, 0.0);

/// Marker for the single persistent player camera.
#[derive(Component)]
pub struct PlayerCamera;

/// Spawns the one persistent camera at [`PLAYER_POS`], looking down -Z.
pub fn spawn_player_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Tonemapping::None,
        DebandDither::Disabled,
        Transform::from_translation(PLAYER_POS).looking_to(-Vec3::Z, Vec3::Y),
        PlayerCamera,
    ));
}
