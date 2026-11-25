//! Tactical Peek Defense: train angle-holding against randomized CS2/Valorant peeks.
pub mod feedback;
pub mod systems;
pub mod types;

use bevy::prelude::*;

use crate::core::hit::CoverWall;
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, TaskInfo, task_playing, task_selected};
use feedback::*;
use systems::*;
use types::*;

pub const INFO: TaskInfo = TaskInfo {
    id: "peek_exercise",
    name: "Peek Defense",
    description: "Hold angles against randomized CS2/Valorant peeks",
    scoring: ScoringMode::Click,
    is_sniper_mode: false,
};

const WALL_EDGE_OFFSET: f32 = WALL_EDGE_X;
const WALL_HALF_WIDTH: f32 = (10.0 - WALL_EDGE_OFFSET) * 0.5;
const WALL_HALF_EXTENTS: Vec3 = Vec3::new(WALL_HALF_WIDTH, 3.0, 0.25);
const LEFT_WALL_X: f32 = -(WALL_EDGE_OFFSET + WALL_HALF_WIDTH);
const RIGHT_WALL_X: f32 = WALL_EDGE_OFFSET + WALL_HALF_WIDTH;
const WALL_Y: f32 = 3.0;

pub fn plugin() -> impl Plugin {
    PeekExercisePlugin
}

struct PeekExercisePlugin;

impl Plugin for PeekExercisePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PeekExerciseState>()
            .init_resource::<PeekRng>()
            .add_systems(
                OnEnter(AppState::Countdown),
                setup.run_if(task_selected(INFO.id)),
            )
            .add_systems(
                Update,
                (
                    spawn_and_move_peeks,
                    handle_shots,
                    sync_target_mesh_material,
                )
                    .run_if(task_playing(INFO.id)),
            )
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<PeekExerciseState>,
    mut rng: ResMut<PeekRng>,
) {
    let target_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.05, 0.06, 0.08),
        perceptual_roughness: 0.20,
        metallic: 0.25,
        reflectance: 0.85,
        ..default()
    });
    let headshot_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.95, 0.2),
        emissive: LinearRgba::rgb(0.1, 0.8, 0.2),
        perceptual_roughness: 0.4,
        metallic: 0.0,
        ..default()
    });
    let despawn_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.1, 0.1),
        emissive: LinearRgba::rgb(0.8, 0.1, 0.1),
        perceptual_roughness: 0.4,
        metallic: 0.0,
        ..default()
    });
    commands.insert_resource(PeekMaterials {
        target: target_mat,
        headshot: headshot_mat,
        despawn: despawn_mat,
    });
    let mut depths = vec![WallDepth::Close, WallDepth::Mid, WallDepth::Far];
    rng.shuffle_depths(&mut depths);
    state.current_depth = depths.pop().unwrap_or(WallDepth::Close);
    state.remaining_depths = depths;
    state.first_side_is_left = rng.gen_bool();
    state.peeks_completed_in_set = 0;
    state.cooldown_timer = Timer::from_seconds(1.0, TimerMode::Once);
    state.active_target = None;
    state.remaining_styles.clear();

    let wall_mesh = meshes.add(Cuboid::new(
        WALL_HALF_EXTENTS.x * 2.0,
        WALL_HALF_EXTENTS.y * 2.0,
        WALL_HALF_EXTENTS.z * 2.0,
    ));
    let wall_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.23, 0.26),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });

    let z = state.current_depth.z();

    // Left cover wall (edge at x = -1.0)
    commands.spawn((
        Mesh3d(wall_mesh.clone()),
        MeshMaterial3d(wall_mat.clone()),
        Transform::from_xyz(LEFT_WALL_X, WALL_Y, z),
        CoverWall {
            half_extents: WALL_HALF_EXTENTS,
        },
        LeftWall,
        PeekExerciseEntity,
    ));

    // Right cover wall (edge at x = 1.0)
    commands.spawn((
        Mesh3d(wall_mesh),
        MeshMaterial3d(wall_mat),
        Transform::from_xyz(RIGHT_WALL_X, WALL_Y, z),
        CoverWall {
            half_extents: WALL_HALF_EXTENTS,
        },
        RightWall,
        PeekExerciseEntity,
    ));
}

fn cleanup(
    mut commands: Commands,
    mut state: ResMut<PeekExerciseState>,
    query: Query<Entity, With<PeekExerciseEntity>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    state.active_target = None;
}
