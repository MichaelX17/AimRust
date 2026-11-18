use bevy::prelude::*;

use crate::core::camera::PlayerCamera;
use crate::core::state::AppState;
use crate::props::environment::{spawn_room, RoomMaterials};

/// Marks everything spawned as part of a round's arena (room + lights).
#[derive(Component)]
pub struct ArenaEntity;

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Countdown), spawn_arena)
            .add_systems(OnEnter(AppState::Results), cleanup_arena)
            .add_systems(OnEnter(AppState::Menu), cleanup_arena);
    }
}

fn spawn_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<RoomMaterials>,
    camera: Query<Entity, With<PlayerCamera>>,
) {
    spawn_room(&mut commands, &mut meshes, &materials);

    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: false,
            illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
            ..default()
        },
        Transform::from_xyz(0.0, 8.0, 5.0).looking_at(Vec3::new(0.0, 0.0, -15.0), Vec3::Y),
        ArenaEntity,
    ));

    // Bevy 0.20 ambient light is a per-camera component (it cannot be marked
    // ArenaEntity without creating a second camera), so it is attached to the
    // persistent player camera and removed on cleanup instead.
    if let Ok(camera) = camera.single() {
        commands.entity(camera).insert(AmbientLight {
            color: Color::WHITE,
            brightness: 200.0,
            ..default()
        });
    }
}

fn cleanup_arena(
    mut commands: Commands,
    entities: Query<Entity, With<ArenaEntity>>,
    camera: Query<Entity, With<PlayerCamera>>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    if let Ok(camera) = camera.single() {
        commands.entity(camera).remove::<AmbientLight>();
    }
}
