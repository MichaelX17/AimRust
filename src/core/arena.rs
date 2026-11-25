use bevy::prelude::*;

use crate::core::camera::PlayerCamera;
use crate::core::state::AppState;
use crate::core::task_api::SelectedTask;
use crate::props::environment::{spawn_room, RoomMaterials};
use crate::world::cylinder_arena::spawn_giant_cylinder_arena;
use crate::world::default_arena::spawn_default_arena;
use crate::world::superflicks_arena::spawn_superflicks_arena;
use crate::world::tenz_corridor_arena::spawn_tenz_corridor_arena;

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
    mut std_materials: ResMut<Assets<StandardMaterial>>,
    room_materials: Res<RoomMaterials>,
    selected_task: Res<SelectedTask>,
    camera: Query<Entity, With<PlayerCamera>>,
) {
    match selected_task.0 {
        "cylinder_micro" => {
            spawn_giant_cylinder_arena(&mut commands, &mut meshes, &mut std_materials);
        }
        "superflicks" => {
            spawn_superflicks_arena(&mut commands, &mut meshes, &mut std_materials);
        }
        "vct_tenz" => {
            spawn_tenz_corridor_arena(&mut commands, &mut meshes, &mut std_materials);
        }
        "sandbox"
        | "peek_exercise"
        | "multishot_ultimate"
        | "micro_adjust"
        | "gridshot"
        | "sixshot"
        | "spidershot"
        | "tracking_follow"
        | "tracking_shoot"
        | "flick"
        | "headshot_precision" => {
            spawn_default_arena(&mut commands, &mut meshes, &mut std_materials);
        }
        _ => {
            spawn_room(&mut commands, &mut meshes, &room_materials);

            commands.spawn((
                DirectionalLight {
                    shadow_maps_enabled: false,
                    illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
                    ..default()
                },
                Transform::from_xyz(0.0, 8.0, 5.0).looking_at(Vec3::new(0.0, 0.0, -15.0), Vec3::Y),
                ArenaEntity,
            ));
        }
    }

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
