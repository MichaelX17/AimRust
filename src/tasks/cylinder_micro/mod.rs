//! Cylinder Micro-Adjustment aim task:
//! 6 small stationary targets distributed across the entire giant cylinder room with wide separation.
//! Targets remain stationary and only despawn when hit by the player.

pub mod types;

use bevy::prelude::*;

use crate::core::hit::{Miss, Target, TargetHit, TargetShape};
use crate::core::score::RoundScore;
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, TaskInfo, task_playing, task_selected};
use crate::props::targets::TargetMaterial;
use types::*;

pub const INFO: TaskInfo = TaskInfo {
    id: "cylinder_micro",
    name: "Cylinder Micro",
    description: "Full-room high-verticality target micro-aiming on the giant cylinder",
    scoring: ScoringMode::Click,
    is_sniper_mode: false,
};

pub fn plugin() -> impl Plugin {
    CylinderMicroPlugin
}

struct CylinderMicroPlugin;

impl Plugin for CylinderMicroPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MicroRng>()
            .add_systems(
                OnEnter(AppState::Countdown),
                setup.run_if(task_selected(INFO.id)),
            )
            .add_systems(
                Update,
                (handle_hits, handle_misses).run_if(task_playing(INFO.id)),
            )
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

fn polar_to_cartesian(yaw: f32, y: f32) -> Vec3 {
    Vec3::new(
        CYLINDER_TARGET_RADIUS * yaw.sin(),
        y,
        -CYLINDER_TARGET_RADIUS * yaw.cos(),
    )
}

fn sample_height(rng: &mut MicroRng) -> f32 {
    let roll = rng.gen_f32();
    if roll < 0.50 {
        // 50% in Bottom cylinder (1.5m to 18.0m: eye-level and low elevation)
        rng.gen_range(1.5, 18.0)
    } else if roll < 0.85 {
        // 35% in Mid cylinder (18.0m to 45.0m: comfortable upward elevation)
        rng.gen_range(18.0, 45.0)
    } else {
        // 15% in Upper/Top cylinder (45.0m to 85.0m: steep high-angle flicks)
        rng.gen_range(45.0, 85.0)
    }
}

fn pick_separated_position(existing_positions: &[Vec3], rng: &mut MicroRng) -> Vec3 {
    let mut best_pos = Vec3::ZERO;
    let mut max_min_dist = -1.0f32;

    for _ in 0..30 {
        let yaw = rng.gen_range(0.0, std::f32::consts::TAU);
        let y = sample_height(rng);
        let candidate = polar_to_cartesian(yaw, y);

        if existing_positions.is_empty() {
            return candidate;
        }

        let mut min_dist = f32::INFINITY;
        for &existing in existing_positions {
            let dist = candidate.distance(existing);
            if dist < min_dist {
                min_dist = dist;
            }
        }

        if min_dist >= MIN_TARGET_SEPARATION {
            return candidate;
        }

        if min_dist > max_min_dist {
            max_min_dist = min_dist;
            best_pos = candidate;
        }
    }

    best_pos
}

fn spawn_single_target(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    pos: Vec3,
) -> Entity {
    commands
        .spawn((
            Mesh3d(meshes.add(Sphere::new(TARGET_RADIUS))),
            MeshMaterial3d(material.0.clone()),
            Transform::from_translation(pos),
            Target {
                shape: TargetShape::Sphere {
                    radius: TARGET_RADIUS,
                },
                lifetime: None,
            },
            MicroTarget,
            CylinderMicroEntity,
        ))
        .id()
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut rng: ResMut<MicroRng>,
) {
    let mut spawned_positions = Vec::with_capacity(ACTIVE_TARGET_COUNT);
    for _ in 0..ACTIVE_TARGET_COUNT {
        let pos = pick_separated_position(&spawned_positions, &mut rng);
        spawned_positions.push(pos);
        spawn_single_target(&mut commands, &mut meshes, &material, pos);
    }
}

fn handle_hits(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut hits: MessageReader<TargetHit>,
    mut score: ResMut<RoundScore>,
    mut rng: ResMut<MicroRng>,
    targets: Query<(Entity, &Transform), With<MicroTarget>>,
) {
    for hit in hits.read() {
        let Ok((entity, _)) = targets.get(hit.entity) else {
            continue;
        };

        score.points += 10;
        score.hits += 1;

        // Despawn hit target
        commands.entity(entity).despawn();

        // Collect positions of all remaining active targets
        let mut remaining_positions = Vec::with_capacity(ACTIVE_TARGET_COUNT);
        for (t_ent, transform) in &targets {
            if t_ent != entity {
                remaining_positions.push(transform.translation);
            }
        }

        // Spawn replacement target with large minimum separation
        let pos = pick_separated_position(&remaining_positions, &mut rng);
        spawn_single_target(&mut commands, &mut meshes, &material, pos);
    }
}

fn handle_misses(mut misses: MessageReader<Miss>, mut score: ResMut<RoundScore>) {
    for _ in misses.read() {
        score.points -= 3;
        score.misses += 1;
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<CylinderMicroEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
