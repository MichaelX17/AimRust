//! Flick / Reaction: one short-lived target, respawning after a random delay.
//! Follows the standard AimRust Default arena and target visual conventions.

pub mod types;

use bevy::prelude::*;

use crate::core::camera::PlayerCamera;
use crate::core::hit::{TargetExpired, TargetHit};
use crate::core::hit_zone::ReactionTimer;
use crate::core::input::PlayerLook;
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, TaskInfo, task_playing, task_selected};
use crate::props::targets::{TargetMaterial, spawn_sphere};
use types::*;

pub const INFO: TaskInfo = TaskInfo {
    id: "flick",
    name: "Flick / Reaction",
    description: "Flick to targets that appear for 1.2 s",
    scoring: ScoringMode::Click,
    is_sniper_mode: false,
};

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
                (handle_removals, spawn_after_delay, update_pop_particles)
                    .chain()
                    .run_if(task_playing(INFO.id)),
            )
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

fn random_position(rng: &mut FlickRng) -> Vec3 {
    Vec3::new(
        rng.gen_range(MIN_X, MAX_X),
        rng.gen_range(MIN_Y, MAX_Y),
        PLANE_Z,
    )
}

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
    commands.entity(entity).insert((
        FlickEntity,
        FlickTarget,
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
        FlickEntity,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(WALL_WIDTH + 0.4, 0.002, 0.4))),
        MeshMaterial3d(shadow_mat),
        Transform::from_xyz(0.0, 0.0018, WALL_Z),
        FlickEntity,
    ));
}

fn spawn_pop_particles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    pos: Vec3,
    rng: &mut FlickRng,
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
            FlickPopParticle {
                velocity: Vec3::new(vx, vy, vz),
                timer: Timer::from_seconds(0.14, TimerMode::Once),
            },
            FlickEntity,
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rng: ResMut<FlickRng>,
    mut state: ResMut<FlickState>,
    mut look: ResMut<PlayerLook>,
    mut cameras: Query<&mut Transform, With<PlayerCamera>>,
    targets: Query<Entity, With<FlickEntity>>,
) {
    for entity in &targets {
        commands.entity(entity).despawn();
    }

    *look = PlayerLook::default();
    if let Ok(mut cam_t) = cameras.single_mut() {
        cam_t.translation = Vec3::new(0.0, 1.7, 0.0);
        cam_t.rotation = Quat::IDENTITY;
    }

    spawn_backboard_wall(&mut commands, &mut meshes, &mut materials);

    state.waiting = true;
    state.timer = rng.next_delay();
    state.last_position = None;
}

#[allow(clippy::too_many_arguments)]
fn handle_removals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut rng: ResMut<FlickRng>,
    mut state: ResMut<FlickState>,
    mut hits: MessageReader<TargetHit>,
    mut expired: MessageReader<TargetExpired>,
    targets: Query<(Entity, &Transform), With<FlickTarget>>,
) {
    let mut handled: Vec<Entity> = Vec::new();

    for hit in hits.read() {
        if let Some((entity, transform)) = targets.iter().find(|(entity, _)| *entity == hit.entity) {
            let hit_pos = transform.translation;
            spawn_pop_particles(&mut commands, &mut meshes, &material, hit_pos, &mut rng);
            state.last_position = Some(hit_pos);
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

fn update_pop_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut FlickPopParticle)>,
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
    entities: Query<Entity, With<FlickEntity>>,
    mut state: ResMut<FlickState>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    state.waiting = false;
    state.timer = 0.0;
    state.last_position = None;
}
