//! Spidershot: one target at a time, alternating near/far spawns.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::core::hit::TargetHit;
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, TaskInfo, task_playing, task_selected};
use crate::props::targets::{TargetMaterial, spawn_sphere};

pub const INFO: TaskInfo = TaskInfo {
    id: "spidershot",
    name: "Spidershot",
    description: "One target, alternating near and far",
    scoring: ScoringMode::Click,
};

const RADIUS: f32 = 0.25;
const PLANE_Z: f32 = -15.0;
const CENTER: Vec2 = Vec2::new(0.0, 1.7);
const NEAR_MAX: f32 = 1.0;
const FAR_MIN: f32 = 3.0;
const FAR_MAX: f32 = 5.0;
const X_LIMIT: f32 = 6.0;
const Y_MIN: f32 = 0.5;
const Y_MAX: f32 = 4.5;

#[derive(Component)]
struct SpidershotEntity;

#[derive(Resource, Default)]
struct SpidershotState {
    index: u32,
}

#[derive(Resource)]
struct SpidershotRng(u64);

impl Default for SpidershotRng {
    fn default() -> Self {
        Self(0xD1B5_4A32_D192_ED03)
    }
}

impl SpidershotRng {
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
    SpidershotPlugin
}

struct SpidershotPlugin;

impl Plugin for SpidershotPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpidershotRng>()
            .init_resource::<SpidershotState>()
            .add_systems(
                OnEnter(AppState::Countdown),
                setup.run_if(task_selected(INFO.id)),
            )
            .add_systems(Update, respawn_on_hit.run_if(task_playing(INFO.id)))
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

fn near_position(rng: &mut SpidershotRng) -> Vec3 {
    let angle = rng.next_f32() * TAU;
    let radius = rng.next_f32() * NEAR_MAX;
    Vec3::new(
        CENTER.x + angle.cos() * radius,
        CENTER.y + angle.sin() * radius,
        PLANE_Z,
    )
}

fn far_position(rng: &mut SpidershotRng) -> Vec3 {
    let angle = rng.next_f32() * TAU;
    let radius = FAR_MIN + rng.next_f32() * (FAR_MAX - FAR_MIN);
    Vec3::new(
        (CENTER.x + angle.cos() * radius).clamp(-X_LIMIT, X_LIMIT),
        (CENTER.y + angle.sin() * radius).clamp(Y_MIN, Y_MAX),
        PLANE_Z,
    )
}

/// Spawns the next target; odd indices are near, even indices are far.
fn spawn_next(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    rng: &mut SpidershotRng,
    state: &mut SpidershotState,
) {
    state.index += 1;
    let position = if state.index % 2 == 1 {
        near_position(rng)
    } else {
        far_position(rng)
    };

    let entity = spawn_sphere(commands, meshes, material, position, RADIUS, None);
    commands.entity(entity).insert(SpidershotEntity);
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut rng: ResMut<SpidershotRng>,
    mut state: ResMut<SpidershotState>,
) {
    state.index = 0;
    spawn_next(&mut commands, &mut meshes, &material, &mut rng, &mut state);
}

fn respawn_on_hit(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut rng: ResMut<SpidershotRng>,
    mut state: ResMut<SpidershotState>,
    mut hits: MessageReader<TargetHit>,
    targets: Query<Entity, With<SpidershotEntity>>,
) {
    for hit in hits.read() {
        if targets.iter().any(|entity| entity == hit.entity) {
            commands.entity(hit.entity).despawn();
            spawn_next(&mut commands, &mut meshes, &material, &mut rng, &mut state);
        }
    }
}

fn cleanup(mut commands: Commands, targets: Query<Entity, With<SpidershotEntity>>) {
    for entity in &targets {
        commands.entity(entity).despawn();
    }
}
