//! Headshot (Precision): adaptive head-level precision flicking on the empty Default arena.
//! - Exactly 1 static 3D PBR sphere strictly at eye/head height (Y = 1.7m)
//! - Horizontal head-level sweep along Z = -15.0m (no backboard wall — pure Default map)
//! - Adaptive Precision sizing: shrinks on each consecutive hit down to micro head-dot size,
//!   and grows back slightly on missed shots

use bevy::prelude::*;

use crate::core::camera::{PLAYER_POS, PlayerCamera};
use crate::core::hit::{Miss, Target, TargetHit, TargetShape};
use crate::core::hit_zone::ReactionTimer;
use crate::core::input::PlayerLook;
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, TaskInfo, task_playing, task_selected};
use crate::props::targets::{TargetMaterial, spawn_sphere};

pub const INFO: TaskInfo = TaskInfo {
    id: "headshot_precision",
    name: "Headshot (Precision)",
    description: "Head-level precision flicks; target shrinks on hits and grows on misses",
    scoring: ScoringMode::Click,
    is_sniper_mode: false,
};

const HEAD_Y: f32 = 1.7;
const PLANE_Z: f32 = -15.0;
const X_HALF_SPAN: f32 = 6.5;
const MIN_STEP_X: f32 = 1.6;
const MAX_STEP_X: f32 = 8.2;

const START_RADIUS: f32 = 0.34;
const MIN_RADIUS: f32 = 0.085;
const MAX_RADIUS: f32 = 0.36;
const SHRINK_ON_HIT: f32 = 0.86;
const GROW_ON_MISS: f32 = 1.15;

#[derive(Component)]
struct HeadshotPrecisionEntity;

#[derive(Component)]
struct HeadshotPrecisionTarget {
    spawn_radius: f32,
}

#[derive(Component)]
struct HeadshotPopParticle {
    velocity: Vec3,
    timer: Timer,
}

#[derive(Resource)]
struct HeadshotPrecisionState {
    current_radius: f32,
}

impl Default for HeadshotPrecisionState {
    fn default() -> Self {
        Self {
            current_radius: START_RADIUS,
        }
    }
}

#[derive(Resource)]
struct HeadshotPrecisionRng(u64);

impl Default for HeadshotPrecisionRng {
    fn default() -> Self {
        Self(0x4EA0_5071_C3B9_28D7)
    }
}

impl HeadshotPrecisionRng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn gen_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }

    fn gen_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.gen_f32() * (max - min)
    }
}

pub fn plugin() -> impl Plugin {
    HeadshotPrecisionPlugin
}

struct HeadshotPrecisionPlugin;

impl Plugin for HeadshotPrecisionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HeadshotPrecisionState>()
            .init_resource::<HeadshotPrecisionRng>()
            .add_systems(
                OnEnter(AppState::Countdown),
                setup.run_if(task_selected(INFO.id)),
            )
            .add_systems(
                Update,
                (handle_hits, handle_misses, update_pop_particles)
                    .chain()
                    .run_if(task_playing(INFO.id)),
            )
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

fn pick_next_x(rng: &mut HeadshotPrecisionRng, vacated_x: f32) -> f32 {
    let mut best_x = rng.gen_range(-X_HALF_SPAN, X_HALF_SPAN);
    let mut best_dist = -1.0f32;

    for _ in 0..48 {
        let cand = rng.gen_range(-X_HALF_SPAN, X_HALF_SPAN);
        let dist = (cand - vacated_x).abs();
        if (MIN_STEP_X..=MAX_STEP_X).contains(&dist) {
            return cand;
        }
        if dist > best_dist {
            best_dist = dist;
            best_x = cand;
        }
    }
    best_x
}

fn spawn_head_target(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    x: f32,
    radius: f32,
) {
    let pos = Vec3::new(x, HEAD_Y, PLANE_Z);
    let entity = spawn_sphere(commands, meshes, material, pos, radius, None);
    commands.entity(entity).insert((
        HeadshotPrecisionEntity,
        HeadshotPrecisionTarget {
            spawn_radius: radius,
        },
        ReactionTimer::new(),
    ));
}

fn spawn_pop_particles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    pos: Vec3,
    rng: &mut HeadshotPrecisionRng,
) {
    let particle_mesh = meshes.add(Sphere::new(0.025));
    for _ in 0..6 {
        let vx = rng.gen_range(-1.8, 1.8);
        let vy = rng.gen_range(-1.0, 2.0);
        let vz = rng.gen_range(0.2, 1.8);

        commands.spawn((
            Mesh3d(particle_mesh.clone()),
            MeshMaterial3d(material.0.clone()),
            Transform::from_translation(pos),
            HeadshotPopParticle {
                velocity: Vec3::new(vx, vy, vz),
                timer: Timer::from_seconds(0.14, TimerMode::Once),
            },
            HeadshotPrecisionEntity,
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut state: ResMut<HeadshotPrecisionState>,
    mut look: ResMut<PlayerLook>,
    mut cameras: Query<&mut Transform, With<PlayerCamera>>,
    existing: Query<Entity, With<HeadshotPrecisionEntity>>,
) {
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    *state = HeadshotPrecisionState::default();
    *look = PlayerLook::default();
    if let Ok(mut cam_t) = cameras.single_mut() {
        cam_t.translation = PLAYER_POS;
        cam_t.rotation = Quat::IDENTITY;
    }

    // Center opening head-level target during countdown, matching reference start
    spawn_head_target(&mut commands, &mut meshes, &material, 0.0, state.current_radius);
}

#[allow(clippy::too_many_arguments)]
fn handle_hits(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut state: ResMut<HeadshotPrecisionState>,
    mut rng: ResMut<HeadshotPrecisionRng>,
    mut hits: MessageReader<TargetHit>,
    targets: Query<(Entity, &Transform), With<HeadshotPrecisionTarget>>,
) {
    for hit in hits.read() {
        let Ok((entity, transform)) = targets.get(hit.entity) else {
            continue;
        };
        let hit_pos = transform.translation;
        spawn_pop_particles(&mut commands, &mut meshes, &material, hit_pos, &mut rng);
        commands.entity(entity).despawn();

        state.current_radius = (state.current_radius * SHRINK_ON_HIT).max(MIN_RADIUS);
        let next_x = pick_next_x(&mut rng, hit_pos.x);
        spawn_head_target(&mut commands, &mut meshes, &material, next_x, state.current_radius);
    }
}

fn handle_misses(
    mut misses: MessageReader<Miss>,
    mut state: ResMut<HeadshotPrecisionState>,
    mut targets: Query<(&mut Transform, &mut Target, &HeadshotPrecisionTarget)>,
) {
    let mut missed = false;
    for _ in misses.read() {
        state.current_radius = (state.current_radius * GROW_ON_MISS).min(MAX_RADIUS);
        missed = true;
    }

    if !missed {
        return;
    }

    for (mut transform, mut target, precision) in &mut targets {
        let scale = (state.current_radius / precision.spawn_radius).max(0.05);
        transform.scale = Vec3::splat(scale);
        target.shape = TargetShape::Sphere {
            radius: state.current_radius,
        };
    }
}

fn update_pop_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut HeadshotPopParticle)>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, mut particle) in &mut particles {
        particle.timer.tick(time.delta());
        if particle.timer.is_finished() {
            commands.entity(entity).despawn();
        } else {
            transform.translation += particle.velocity * dt;
            let frac = 1.0 - particle.timer.fraction();
            transform.scale = Vec3::splat(frac);
        }
    }
}

fn cleanup(
    mut commands: Commands,
    entities: Query<Entity, With<HeadshotPrecisionEntity>>,
    mut state: ResMut<HeadshotPrecisionState>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    *state = HeadshotPrecisionState::default();
}
