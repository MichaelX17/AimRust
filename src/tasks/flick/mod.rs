//! Flick / Reaction: one short-lived target, respawning after a random delay.

use bevy::prelude::*;

use crate::core::hit::{TargetExpired, TargetHit};
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, TaskInfo, task_playing, task_selected};
use crate::props::targets::{TargetMaterial, spawn_sphere};

pub const INFO: TaskInfo = TaskInfo {
    id: "flick",
    name: "Flick / Reaction",
    description: "Flick to targets that appear for 1.2 s",
    scoring: ScoringMode::Click,
};

const RADIUS: f32 = 0.25;
const PLANE_Z: f32 = -15.0;
const MIN_X: f32 = -6.0;
const MAX_X: f32 = 6.0;
const MIN_Y: f32 = 0.6;
const MAX_Y: f32 = 4.2;
const LIFETIME: f32 = 1.2;
const MIN_DELAY: f32 = 0.3;
const MAX_DELAY: f32 = 1.0;
const MIN_RESPAWN_DISTANCE: f32 = 1.0;
const RESPAWN_ATTEMPTS: usize = 32;

#[derive(Component)]
struct FlickEntity;

#[derive(Resource, Default)]
struct FlickState {
    waiting: bool,
    timer: f32,
    last_position: Option<Vec3>,
}

#[derive(Resource)]
struct FlickRng(u64);

impl Default for FlickRng {
    fn default() -> Self {
        Self(0xE703_7ED1_A0B4_28DB)
    }
}

impl FlickRng {
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

    fn next_delay(&mut self) -> f32 {
        MIN_DELAY + self.next_f32() * (MAX_DELAY - MIN_DELAY)
    }
}

pub fn plugin() -> impl Plugin {
    FlickPlugin
}

struct FlickPlugin;

impl Plugin for FlickPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FlickRng>()
            .init_resource::<FlickState>()
            .add_systems(
                OnEnter(AppState::Countdown),
                setup.run_if(task_selected(INFO.id)),
            )
            .add_systems(
                Update,
                (handle_removals, spawn_after_delay)
                    .chain()
                    .run_if(task_playing(INFO.id)),
            )
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

fn random_position(rng: &mut FlickRng) -> Vec3 {
    Vec3::new(
        MIN_X + rng.next_f32() * (MAX_X - MIN_X),
        MIN_Y + rng.next_f32() * (MAX_Y - MIN_Y),
        PLANE_Z,
    )
}

/// Random position at least [`MIN_RESPAWN_DISTANCE`] from `avoid`.
fn random_position_away_from(rng: &mut FlickRng, avoid: Vec3) -> Vec3 {
    let mut fallback = random_position(rng);
    let mut fallback_distance = f32::MIN;

    for _ in 0..RESPAWN_ATTEMPTS {
        let candidate = random_position(rng);
        let distance = candidate.distance(avoid);
        if distance >= MIN_RESPAWN_DISTANCE {
            return candidate;
        }
        if distance > fallback_distance {
            fallback_distance = distance;
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
    let entity = spawn_sphere(commands, meshes, material, position, RADIUS, Some(LIFETIME));
    commands.entity(entity).insert(FlickEntity);
}

fn setup(
    mut rng: ResMut<FlickRng>,
    mut state: ResMut<FlickState>,
    targets: Query<Entity, With<FlickEntity>>,
    mut commands: Commands,
) {
    for entity in &targets {
        commands.entity(entity).despawn();
    }
    state.waiting = true;
    state.timer = rng.next_delay();
    state.last_position = None;
}

fn handle_removals(
    mut commands: Commands,
    mut rng: ResMut<FlickRng>,
    mut state: ResMut<FlickState>,
    mut hits: MessageReader<TargetHit>,
    mut expired: MessageReader<TargetExpired>,
    targets: Query<(Entity, &Transform), With<FlickEntity>>,
) {
    let mut handled: Vec<Entity> = Vec::new();

    for hit in hits.read() {
        if let Some((entity, transform)) = targets.iter().find(|(entity, _)| *entity == hit.entity) {
            state.last_position = Some(transform.translation);
            commands.entity(entity).despawn();
            handled.push(entity);
        }
    }
    for event in expired.read() {
        if handled.contains(&event.entity) {
            continue;
        }
        if let Some((entity, transform)) = targets.iter().find(|(entity, _)| *entity == event.entity)
        {
            state.last_position = Some(transform.translation);
            commands.entity(entity).despawn();
            handled.push(entity);
        }
    }

    if !handled.is_empty() {
        state.waiting = true;
        state.timer = rng.next_delay();
    }
}

fn spawn_after_delay(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut rng: ResMut<FlickRng>,
    time: Res<Time>,
    mut state: ResMut<FlickState>,
) {
    if !state.waiting {
        return;
    }
    state.timer -= time.delta_secs();
    if state.timer > 0.0 {
        return;
    }

    let position = match state.last_position {
        Some(last) => random_position_away_from(&mut rng, last),
        None => random_position(&mut rng),
    };
    spawn_target(&mut commands, &mut meshes, &material, position);
    state.waiting = false;
}

fn cleanup(mut commands: Commands, targets: Query<Entity, With<FlickEntity>>) {
    for entity in &targets {
        commands.entity(entity).despawn();
    }
}
