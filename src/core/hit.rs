//! Target/hitscan API. Consumed by task modules from Task_10.
#![allow(dead_code)]

use bevy::prelude::*;

use crate::core::camera::PlayerCamera;
use crate::core::input::Shot;
use crate::core::state::AppState;

#[derive(Clone, Copy, Debug)]
pub enum TargetShape {
    Sphere { radius: f32 },
    /// Axis-aligned box (no rotation).
    Box { half_extents: Vec3 },
}

#[derive(Component)]
pub struct Target {
    pub shape: TargetShape,
    pub lifetime: Option<f32>,
}

/// Marks a target that already emitted [`TargetExpired`].
#[derive(Component)]
pub struct Expired;

#[derive(Message)]
pub struct TargetHit {
    pub entity: Entity,
}

#[derive(Message)]
pub struct Miss;

#[derive(Message)]
pub struct TargetExpired {
    pub entity: Entity,
}

/// Target currently under the crosshair.
#[derive(Resource, Default)]
pub struct AimingAt(pub Option<Entity>);

pub struct HitPlugin;

impl Plugin for HitPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TargetHit>()
            .add_message::<Miss>()
            .add_message::<TargetExpired>()
            .init_resource::<AimingAt>()
            .add_systems(
                Update,
                (expire_targets, update_aiming, consume_shots)
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

/// Analytic ray/sphere intersection; returns the distance along `dir`.
fn ray_sphere(origin: Vec3, dir: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let oc = center - origin;
    let tca = oc.dot(dir);
    if tca < 0.0 {
        return None;
    }
    let d2 = oc.length_squared() - tca * tca;
    let r2 = radius * radius;
    if d2 > r2 {
        return None;
    }
    let t = tca - (r2 - d2).sqrt();
    (t >= 0.0).then_some(t)
}

/// Analytic ray/AABB intersection (slab method); returns the entry distance.
fn ray_aabb(origin: Vec3, dir: Vec3, center: Vec3, half_extents: Vec3) -> Option<f32> {
    let mut t_min = 0.0f32;
    let mut t_max = f32::INFINITY;
    let min = center - half_extents;
    let max = center + half_extents;

    for axis in 0..3 {
        let d = dir[axis];
        let o = origin[axis];
        if d.abs() < 1e-6 {
            if o < min[axis] || o > max[axis] {
                return None;
            }
            continue;
        }
        let mut t1 = (min[axis] - o) / d;
        let mut t2 = (max[axis] - o) / d;
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }
        t_min = t_min.max(t1);
        t_max = t_max.min(t2);
        if t_min > t_max {
            return None;
        }
    }
    (t_max >= 0.0).then_some(t_min)
}

fn ray_hit(origin: Vec3, dir: Vec3, center: Vec3, shape: &TargetShape) -> Option<f32> {
    match *shape {
        TargetShape::Sphere { radius } => ray_sphere(origin, dir, center, radius),
        TargetShape::Box { half_extents } => ray_aabb(origin, dir, center, half_extents),
    }
}

/// Keeps [`AimingAt`] pointing at the nearest target under the crosshair.
fn update_aiming(
    camera: Query<&GlobalTransform, With<PlayerCamera>>,
    targets: Query<(Entity, &Target, &GlobalTransform)>,
    mut aiming: ResMut<AimingAt>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    let origin = camera.translation();
    let dir = *camera.forward();

    let mut best: Option<(f32, Entity)> = None;
    for (entity, target, transform) in &targets {
        let center = transform.translation();
        if let Some(distance) = ray_hit(origin, dir, center, &target.shape)
            && best.is_none_or(|(best_distance, _)| distance < best_distance)
        {
            best = Some((distance, entity));
        }
    }
    aiming.0 = best.map(|(_, entity)| entity);
}

/// Turns each shot into a hit or a miss based on the current aim.
fn consume_shots(
    mut shots: MessageReader<Shot>,
    aiming: Res<AimingAt>,
    mut hits: MessageWriter<TargetHit>,
    mut misses: MessageWriter<Miss>,
) {
    for _ in shots.read() {
        if let Some(entity) = aiming.0 {
            hits.write(TargetHit { entity });
        } else {
            misses.write(Miss);
        }
    }
}

/// Ticks target lifetimes; emits [`TargetExpired`] once per target.
fn expire_targets(
    time: Res<Time>,
    mut commands: Commands,
    mut targets: Query<(Entity, &mut Target, Has<Expired>)>,
    mut expired: MessageWriter<TargetExpired>,
) {
    for (entity, mut target, is_expired) in &mut targets {
        if is_expired {
            continue;
        }
        let Some(lifetime) = target.lifetime.as_mut() else {
            continue;
        };
        *lifetime -= time.delta_secs();
        if *lifetime <= 0.0 {
            commands.entity(entity).insert(Expired);
            expired.write(TargetExpired { entity });
        }
    }
}
