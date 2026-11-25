//! Spidershot: one target at a time, alternating near/far spawns.
//! Follows the standard AimRust Default arena and target visual conventions.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::core::camera::PlayerCamera;
use crate::core::hit::TargetHit;
use crate::core::hit_zone::ReactionTimer;
use crate::core::input::PlayerLook;
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, TaskInfo, task_playing, task_selected};
use crate::props::targets::{TargetMaterial, spawn_sphere};

pub const INFO: TaskInfo = TaskInfo {
    id: "spidershot",
    name: "Spidershot",
    description: "One target, alternating near and far",
    scoring: ScoringMode::Click,
    is_sniper_mode: false,
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

// Backboard wall geometry on the Default arena
const WALL_WIDTH: f32 = 18.0;
const WALL_HEIGHT: f32 = 7.5;
const WALL_DEPTH: f32 = 0.3;
const WALL_Z: f32 = -15.6;

#[derive(Component)]
struct SpidershotEntity;

#[derive(Component)]
struct SpidershotTarget;

#[derive(Component)]
struct SpidershotPopParticle {
    velocity: Vec3,
    timer: Timer,
}

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

    fn gen_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }

    fn gen_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.gen_f32() * (max - min)
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
            .add_systems(
                Update,
                (respawn_on_hit, update_pop_particles).run_if(task_playing(INFO.id)),
            )
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

fn near_position(rng: &mut SpidershotRng) -> Vec3 {
    let angle = rng.gen_f32() * TAU;
    let radius = rng.gen_f32() * NEAR_MAX;
    Vec3::new(
        CENTER.x + angle.cos() * radius,
        CENTER.y + angle.sin() * radius,
        PLANE_Z,
    )
}

fn far_position(rng: &mut SpidershotRng) -> Vec3 {
    let angle = rng.gen_f32() * TAU;
    let radius = rng.gen_range(FAR_MIN, FAR_MAX);
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
    commands.entity(entity).insert((
        SpidershotEntity,
        SpidershotTarget,
        ReactionTimer::new(),
    ));
}

fn spawn_backboard_wall(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let backboard_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(0xD6, 0xD9, 0xDE),
        perceptual_roughness: 0.92,
        metallic: 0.01,
        reflectance: 0.2,
        ..default()
    });

    let shadow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba_u8(0x0a, 0x0c, 0x0f, 0x99),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 1.0,
        metallic: 0.0,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(WALL_WIDTH, WALL_HEIGHT, WALL_DEPTH))),
        MeshMaterial3d(backboard_mat),
        Transform::from_xyz(0.0, WALL_HEIGHT * 0.5, WALL_Z),
        SpidershotEntity,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(WALL_WIDTH + 0.4, 0.002, 0.4))),
        MeshMaterial3d(shadow_mat),
        Transform::from_xyz(0.0, 0.0018, WALL_Z),
        SpidershotEntity,
    ));
}

fn spawn_pop_particles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    pos: Vec3,
    rng: &mut SpidershotRng,
) {
    let particle_mesh = meshes.add(Sphere::new(0.03));
    for _ in 0..6 {
        let vx = rng.gen_range(-1.8, 1.8);
        let vy = rng.gen_range(-1.0, 2.0);
        let vz = rng.gen_range(0.2, 1.8);

        commands.spawn((
            Mesh3d(particle_mesh.clone()),
            MeshMaterial3d(material.0.clone()),
            Transform::from_translation(pos),
            SpidershotPopParticle {
                velocity: Vec3::new(vx, vy, vz),
                timer: Timer::from_seconds(0.14, TimerMode::Once),
            },
            SpidershotEntity,
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material: Res<TargetMaterial>,
    mut rng: ResMut<SpidershotRng>,
    mut state: ResMut<SpidershotState>,
    mut look: ResMut<PlayerLook>,
    mut cameras: Query<&mut Transform, With<PlayerCamera>>,
) {
    *look = PlayerLook::default();
    if let Ok(mut cam_t) = cameras.single_mut() {
        cam_t.translation = Vec3::new(0.0, 1.7, 0.0);
        cam_t.rotation = Quat::IDENTITY;
    }

    spawn_backboard_wall(&mut commands, &mut meshes, &mut materials);

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
    targets: Query<(Entity, &Transform), With<SpidershotTarget>>,
) {
    for hit in hits.read() {
        let Some((_, hit_transform)) = targets.iter().find(|(entity, _)| *entity == hit.entity)
        else {
            continue;
        };
        let hit_pos = hit_transform.translation;

        spawn_pop_particles(&mut commands, &mut meshes, &material, hit_pos, &mut rng);
        commands.entity(hit.entity).despawn();
        spawn_next(&mut commands, &mut meshes, &material, &mut rng, &mut state);
    }
}

fn update_pop_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut SpidershotPopParticle)>,
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
    entities: Query<Entity, With<SpidershotEntity>>,
    mut state: ResMut<SpidershotState>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    state.index = 0;
}
