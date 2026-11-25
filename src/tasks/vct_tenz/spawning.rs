//! Portal target spawning and hit pop-particle helpers for VCT NA - TenZ.

use bevy::prelude::*;

use crate::core::hit_zone::ReactionTimer;
use crate::props::targets::{TargetMaterial, spawn_sphere};
use crate::world::tenz_corridor_arena::SPAWN_Z;

use super::types::*;

/// Samples an `(X, Y)` coordinate inside the far neon portal separated from `avoid`.
pub fn sample_portal_xy(rng: &mut TenZRng, avoid: &[Vec2]) -> Vec2 {
    let mut best = Vec2::new(
        rng.gen_range(-SPAWN_X_HALF, SPAWN_X_HALF),
        rng.gen_range(SPAWN_Y_MIN, SPAWN_Y_MAX),
    );
    let mut best_sep = -1.0f32;

    for _ in 0..36 {
        let cand = Vec2::new(
            rng.gen_range(-SPAWN_X_HALF, SPAWN_X_HALF),
            rng.gen_range(SPAWN_Y_MIN, SPAWN_Y_MAX),
        );
        let sep = avoid
            .iter()
            .map(|p| cand.distance(*p))
            .fold(f32::INFINITY, f32::min);
        if sep >= MIN_PORTAL_SEP {
            return cand;
        }
        if sep > best_sep {
            best_sep = sep;
            best = cand;
        }
    }
    best
}

pub fn spawn_orb(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    xy: Vec2,
    speed: f32,
) {
    let pos = Vec3::new(xy.x, xy.y, SPAWN_Z);
    let entity = spawn_sphere(commands, meshes, material, pos, TARGET_RADIUS, None);
    commands.entity(entity).insert((
        TenZEntity,
        TenZTarget { speed },
        ReactionTimer::new(),
    ));
}

pub fn spawn_pop_particles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    pos: Vec3,
    rng: &mut TenZRng,
) {
    let particle_mesh = meshes.add(Sphere::new(0.032));
    for _ in 0..6 {
        let vx = rng.gen_range(-2.2, 2.2);
        let vy = rng.gen_range(-1.3, 2.3);
        let vz = rng.gen_range(-1.5, 2.2);

        commands.spawn((
            Mesh3d(particle_mesh.clone()),
            MeshMaterial3d(material.0.clone()),
            Transform::from_translation(pos),
            TenZPopParticle {
                velocity: Vec3::new(vx, vy, vz),
                timer: Timer::from_seconds(0.14, TimerMode::Once),
            },
            TenZEntity,
        ));
    }
}

pub fn update_pop_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut TenZPopParticle)>,
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
