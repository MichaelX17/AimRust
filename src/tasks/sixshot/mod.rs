//! Sixshot: six sphere targets, always visible, respawn on hit.
//! Follows the standard AimRust Default arena and target visual conventions.

use bevy::prelude::*;

use crate::core::camera::PlayerCamera;
use crate::core::hit::TargetHit;
use crate::core::hit_zone::ReactionTimer;
use crate::core::input::PlayerLook;
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
const RADIUS: f32 = 0.20;
const PLANE_Z: f32 = -15.0;
const X_RANGE: f32 = 4.0;
const Y_MIN: f32 = 0.8;
const Y_RANGE: f32 = 3.0;
const MIN_DISTANCE: f32 = 1.0;
const MAX_ATTEMPTS: usize = 200;

// Backboard wall geometry on the Default arena
const WALL_WIDTH: f32 = 14.0;
const WALL_HEIGHT: f32 = 6.5;
const WALL_DEPTH: f32 = 0.3;
const WALL_Z: f32 = -15.6;

#[derive(Component)]
struct SixshotEntity;

#[derive(Component)]
struct SixshotPopParticle {
    velocity: Vec3,
    timer: Timer,
}

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

    fn gen_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }

    fn gen_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.gen_f32() * (max - min)
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
            .add_systems(
                Update,
                (respawn_on_hit, update_pop_particles).run_if(task_playing(INFO.id)),
            )
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

fn sample_position(rng: &mut SixshotRng) -> Vec3 {
    Vec3::new(
        rng.gen_range(-X_RANGE, X_RANGE),
        rng.gen_range(Y_MIN, Y_MIN + Y_RANGE),
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
    commands
        .entity(entity)
        .insert((SixshotEntity, ReactionTimer::new()));
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
        SixshotEntity,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(WALL_WIDTH + 0.4, 0.002, 0.4))),
        MeshMaterial3d(shadow_mat),
        Transform::from_xyz(0.0, 0.0018, WALL_Z),
        SixshotEntity,
    ));
}

fn spawn_pop_particles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    pos: Vec3,
    rng: &mut SixshotRng,
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
            SixshotPopParticle {
                velocity: Vec3::new(vx, vy, vz),
                timer: Timer::from_seconds(0.14, TimerMode::Once),
            },
            SixshotEntity,
        ));
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material: Res<TargetMaterial>,
    mut rng: ResMut<SixshotRng>,
    mut look: ResMut<PlayerLook>,
    mut cameras: Query<&mut Transform, With<PlayerCamera>>,
) {
    *look = PlayerLook::default();
    if let Ok(mut cam_t) = cameras.single_mut() {
        cam_t.translation = Vec3::new(0.0, 1.7, 0.0);
        cam_t.rotation = Quat::IDENTITY;
    }

    spawn_backboard_wall(&mut commands, &mut meshes, &mut materials);

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

        spawn_pop_particles(&mut commands, &mut meshes, &material, vacated, &mut rng);

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

fn update_pop_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut SixshotPopParticle)>,
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

fn cleanup(mut commands: Commands, entities: Query<Entity, With<SixshotEntity>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
