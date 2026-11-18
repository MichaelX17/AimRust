//! Tracking Shoot: moving target with a 4 s lifetime; hit or expiry respawns it.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::core::hit::{TargetExpired, TargetHit};
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, TaskInfo, task_playing, task_selected};
use crate::props::targets::{TargetMaterial, spawn_sphere};

pub const INFO: TaskInfo = TaskInfo {
    id: "tracking_shoot",
    name: "Tracking: Shoot",
    description: "Track and shoot a short-lived moving target",
    scoring: ScoringMode::Click,
};

const RADIUS: f32 = 0.3;
const PLANE_Z: f32 = -15.0;
const SPEED: f32 = 4.0;
const LIFETIME: f32 = 4.0;
const MIN_X: f32 = -5.0;
const MAX_X: f32 = 5.0;
const MIN_Y: f32 = 0.8;
const MAX_Y: f32 = 4.0;
const MIN_CHANGE: f32 = 0.5;
const MAX_CHANGE: f32 = 1.5;
const STEER_RATE: f32 = 6.0;
const MIN_RESPAWN_DISTANCE: f32 = 1.0;
const RESPAWN_ATTEMPTS: usize = 32;

#[derive(Component)]
struct TrackingShootEntity;

#[derive(Component)]
struct TrackingShoot {
    velocity: Vec2,
    target_velocity: Vec2,
    change_in: f32,
}

#[derive(Resource)]
struct TrackingShootRng(u64);

impl Default for TrackingShootRng {
    fn default() -> Self {
        Self(0xA076_1D64_78BD_642F)
    }
}

impl TrackingShootRng {
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

    fn next_direction(&mut self) -> Vec2 {
        let angle = self.next_f32() * TAU;
        Vec2::new(angle.cos(), angle.sin())
    }
}

pub fn plugin() -> impl Plugin {
    TrackingShootPlugin
}

struct TrackingShootPlugin;

impl Plugin for TrackingShootPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrackingShootRng>()
            .add_systems(
                OnEnter(AppState::Countdown),
                setup.run_if(task_selected(INFO.id)),
            )
            .add_systems(
                Update,
                (movement, respawn_on_events).run_if(task_playing(INFO.id)),
            )
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

fn random_position(rng: &mut TrackingShootRng) -> Vec3 {
    Vec3::new(
        MIN_X + rng.next_f32() * (MAX_X - MIN_X),
        MIN_Y + rng.next_f32() * (MAX_Y - MIN_Y),
        PLANE_Z,
    )
}

/// Random position at least [`MIN_RESPAWN_DISTANCE`] from `avoid`; falls back to
/// the farthest sampled candidate.
fn random_position_away_from(rng: &mut TrackingShootRng, avoid: Vec3) -> Vec3 {
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
    rng: &mut TrackingShootRng,
    position: Vec3,
) {
    let direction = rng.next_direction() * SPEED;
    let change_in = MIN_CHANGE + rng.next_f32() * (MAX_CHANGE - MIN_CHANGE);

    let entity = spawn_sphere(commands, meshes, material, position, RADIUS, Some(LIFETIME));
    commands.entity(entity).insert((
        TrackingShootEntity,
        TrackingShoot {
            velocity: direction,
            target_velocity: direction,
            change_in,
        },
    ));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut rng: ResMut<TrackingShootRng>,
) {
    let position = random_position(&mut rng);
    spawn_target(&mut commands, &mut meshes, &material, &mut rng, position);
}

fn movement(
    time: Res<Time>,
    mut rng: ResMut<TrackingShootRng>,
    mut targets: Query<(&mut Transform, &mut TrackingShoot), With<TrackingShootEntity>>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut target) in &mut targets {
        target.change_in -= dt;
        if target.change_in <= 0.0 {
            target.target_velocity = rng.next_direction() * SPEED;
            target.change_in = MIN_CHANGE + rng.next_f32() * (MAX_CHANGE - MIN_CHANGE);
        }

        let alpha = (dt * STEER_RATE).min(1.0);
        target.velocity = target.velocity.lerp(target.target_velocity, alpha);

        let mut position = transform.translation;
        position.x += target.velocity.x * dt;
        position.y += target.velocity.y * dt;

        if position.x < MIN_X {
            position.x = MIN_X;
            target.velocity.x = target.velocity.x.abs();
            target.target_velocity.x = target.target_velocity.x.abs();
        } else if position.x > MAX_X {
            position.x = MAX_X;
            target.velocity.x = -target.velocity.x.abs();
            target.target_velocity.x = -target.target_velocity.x.abs();
        }

        if position.y < MIN_Y {
            position.y = MIN_Y;
            target.velocity.y = target.velocity.y.abs();
            target.target_velocity.y = target.target_velocity.y.abs();
        } else if position.y > MAX_Y {
            position.y = MAX_Y;
            target.velocity.y = -target.velocity.y.abs();
            target.target_velocity.y = -target.target_velocity.y.abs();
        }

        position.z = PLANE_Z;
        transform.translation = position;
    }
}

fn respawn_on_events(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut rng: ResMut<TrackingShootRng>,
    mut hits: MessageReader<TargetHit>,
    mut expired: MessageReader<TargetExpired>,
    targets: Query<(Entity, &Transform), With<TrackingShootEntity>>,
) {
    let mut handled: Vec<Entity> = Vec::new();

    for hit in hits.read() {
        if let Some((entity, transform)) = targets.iter().find(|(entity, _)| *entity == hit.entity) {
            let position = random_position_away_from(&mut rng, transform.translation);
            commands.entity(entity).despawn();
            spawn_target(&mut commands, &mut meshes, &material, &mut rng, position);
            handled.push(entity);
        }
    }

    for event in expired.read() {
        if handled.contains(&event.entity) {
            continue;
        }
        if let Some((entity, transform)) = targets.iter().find(|(entity, _)| *entity == event.entity)
        {
            let position = random_position_away_from(&mut rng, transform.translation);
            commands.entity(entity).despawn();
            spawn_target(&mut commands, &mut meshes, &material, &mut rng, position);
            handled.push(entity);
        }
    }
}

fn cleanup(mut commands: Commands, targets: Query<Entity, With<TrackingShootEntity>>) {
    for entity in &targets {
        commands.entity(entity).despawn();
    }
}
