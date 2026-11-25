//! Tracking Shoot: moving target with a 4 s lifetime; hit or expiry respawns it.
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
    id: "tracking_shoot",
    name: "Tracking: Shoot",
    description: "Track and shoot a short-lived moving target",
    scoring: ScoringMode::Click,
    is_sniper_mode: false,
};

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
                (movement, respawn_on_events, update_pop_particles).run_if(task_playing(INFO.id)),
            )
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

fn random_position(rng: &mut TrackingShootRng) -> Vec3 {
    Vec3::new(
        rng.gen_range(MIN_X, MAX_X),
        rng.gen_range(MIN_Y, MAX_Y),
        PLANE_Z,
    )
}

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
    let change_in = rng.gen_range(MIN_CHANGE, MAX_CHANGE);

    let entity = spawn_sphere(commands, meshes, material, position, RADIUS, Some(LIFETIME));
    commands.entity(entity).insert((
        TrackingShootEntity,
        TrackingShootTarget,
        ReactionTimer::new(),
        TrackingShoot {
            velocity: direction,
            target_velocity: direction,
            change_in,
        },
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
        TrackingShootEntity,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(WALL_WIDTH + 0.4, 0.002, 0.4))),
        MeshMaterial3d(shadow_mat),
        Transform::from_xyz(0.0, 0.0018, WALL_Z),
        TrackingShootEntity,
    ));
}

fn spawn_pop_particles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    pos: Vec3,
    rng: &mut TrackingShootRng,
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
            TrackingShootPopParticle {
                velocity: Vec3::new(vx, vy, vz),
                timer: Timer::from_seconds(0.14, TimerMode::Once),
            },
            TrackingShootEntity,
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material: Res<TargetMaterial>,
    mut rng: ResMut<TrackingShootRng>,
    mut look: ResMut<PlayerLook>,
    mut cameras: Query<&mut Transform, With<PlayerCamera>>,
) {
    *look = PlayerLook::default();
    if let Ok(mut cam_t) = cameras.single_mut() {
        cam_t.translation = Vec3::new(0.0, 1.7, 0.0);
        cam_t.rotation = Quat::IDENTITY;
    }

    spawn_backboard_wall(&mut commands, &mut meshes, &mut materials);

    let position = random_position(&mut rng);
    spawn_target(&mut commands, &mut meshes, &material, &mut rng, position);
}

fn movement(
    time: Res<Time>,
    mut rng: ResMut<TrackingShootRng>,
    mut targets: Query<(&mut Transform, &mut TrackingShoot), With<TrackingShootTarget>>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut target) in &mut targets {
        target.change_in -= dt;
        if target.change_in <= 0.0 {
            target.target_velocity = rng.next_direction() * SPEED;
            target.change_in = rng.gen_range(MIN_CHANGE, MAX_CHANGE);
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
    targets: Query<(Entity, &Transform), With<TrackingShootTarget>>,
) {
    let mut handled: Vec<Entity> = Vec::new();

    for hit in hits.read() {
        if let Some((entity, transform)) = targets.iter().find(|(entity, _)| *entity == hit.entity) {
            let hit_pos = transform.translation;
            spawn_pop_particles(&mut commands, &mut meshes, &material, hit_pos, &mut rng);
            let position = random_position_away_from(&mut rng, hit_pos);
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

fn update_pop_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut TrackingShootPopParticle)>,
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

fn cleanup(mut commands: Commands, entities: Query<Entity, With<TrackingShootEntity>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
