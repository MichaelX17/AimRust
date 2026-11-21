//! Sixshot: six sphere targets, always visible, respawn on hit.

use bevy::prelude::*;

use crate::core::hit::TargetHit;
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, TaskInfo, task_playing, task_selected};
use crate::props::targets::{TargetMaterial, spawn_sphere};

pub const INFO: TaskInfo = TaskInfo {
    id: "sixshot",
    name: "Sixshot",
    description: "Six targets; hit one, another appears",
    scoring: ScoringMode::Click,
    is_sniper_mode: false,
};

const TARGET_COUNT: usize = 6;
const RADIUS: f32 = 0.2;
const PLANE_Z: f32 = -15.0;
const X_RANGE: f32 = 4.0;
const Y_MIN: f32 = 0.8;
const Y_RANGE: f32 = 3.0;
const MIN_DISTANCE: f32 = 1.0;
const MAX_ATTEMPTS: usize = 200;

#[derive(Component)]
struct SixshotEntity;

#[derive(Resource)]
struct SixshotRng(u64);

impl Default for SixshotRng {
    fn default() -> Self {
        Self(0x9E37_79B9_7F4A_7C15)
    }
}

impl SixshotRng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn next_f32(&mut self) -> f32 {
        ((self.next_u64() >> 40) as u32) as f32 / 16_777_216.0
    }
}

pub fn plugin() -> impl Plugin {
    SixshotPlugin
}

struct SixshotPlugin;

impl Plugin for SixshotPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SixshotRng>()
            .add_systems(
                OnEnter(AppState::Countdown),
                setup.run_if(task_selected(INFO.id)),
            )
            .add_systems(Update, respawn_on_hit.run_if(task_playing(INFO.id)))
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

fn sample_position(rng: &mut SixshotRng) -> Vec3 {
    Vec3::new(
        -X_RANGE + rng.next_f32() * X_RANGE * 2.0,
        Y_MIN + rng.next_f32() * Y_RANGE,
        PLANE_Z,
    )
}

/// Samples a position at least [`MIN_DISTANCE`] away from every occupied point.
/// Falls back to the best sampled candidate if no perfect one is found.
fn random_valid_position(rng: &mut SixshotRng, occupied: &[Vec3]) -> Vec3 {
    let mut fallback = sample_position(rng);
    let mut fallback_min = f32::MIN;

    for _ in 0..MAX_ATTEMPTS {
        let candidate = sample_position(rng);
        let min_distance = occupied
            .iter()
            .map(|point| candidate.distance(*point))
            .fold(f32::INFINITY, f32::min);
        if min_distance >= MIN_DISTANCE {
            return candidate;
        }
        if min_distance > fallback_min {
            fallback_min = min_distance;
            fallback = candidate;
        }
    }
    fallback
}

fn spawn_target(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    position: Vec3,
) {
    let entity = spawn_sphere(commands, meshes, material, position, RADIUS, None);
    commands.entity(entity).insert(SixshotEntity);
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut rng: ResMut<SixshotRng>,
) {
    let mut occupied: Vec<Vec3> = Vec::with_capacity(TARGET_COUNT);
    for _ in 0..TARGET_COUNT {
        let position = random_valid_position(&mut rng, &occupied);
        occupied.push(position);
        spawn_target(&mut commands, &mut meshes, &material, position);
    }
}

fn respawn_on_hit(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut rng: ResMut<SixshotRng>,
    mut hits: MessageReader<TargetHit>,
    targets: Query<(Entity, &Transform), With<SixshotEntity>>,
) {
    for hit in hits.read() {
        let Some((_, vacated_transform)) = targets.iter().find(|(entity, _)| *entity == hit.entity)
        else {
            continue;
        };
        let vacated = vacated_transform.translation;

        // Keep distance from the remaining targets and from the just-vacated
        // spot: a target never respawns where it was hit.
        let mut occupied: Vec<Vec3> = targets
            .iter()
            .filter(|(entity, _)| *entity != hit.entity)
            .map(|(_, transform)| transform.translation)
            .collect();
        occupied.push(vacated);

        let position = random_valid_position(&mut rng, &occupied);

        commands.entity(hit.entity).despawn();
        spawn_target(&mut commands, &mut meshes, &material, position);
    }
}

fn cleanup(mut commands: Commands, targets: Query<Entity, With<SixshotEntity>>) {
    for entity in &targets {
        commands.entity(entity).despawn();
    }
}
