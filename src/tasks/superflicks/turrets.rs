//! Level 4 (Phase 4) floor-turret ballistic projectile launcher and physics.
//! Launches orbs strictly from turrets in front of the player's current look direction,
//! arcing left->right or right->left across the view, and despawns silently on floor contact.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use crate::core::camera::PlayerCamera;
use crate::core::hit_zone::ReactionTimer;
use crate::props::targets::{TargetMaterial, spawn_sphere};
use crate::world::superflicks_arena::{ATRIUM_TURRET_COUNT, LEVEL_Y, atrium_turret_top};

use super::types::*;

/// Launches one sweeping left->right (`sweep_dir > 0`) or right->left (`sweep_dir < 0`)
/// parabolic projectile orb from Level 4 perimeter floor turret `turret_idx`.
pub fn spawn_turret_projectile(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    rng: &mut SuperFlicksRng,
    turret_idx: usize,
    sweep_dir: f32,
) {
    let idx = turret_idx % ATRIUM_TURRET_COUNT;
    let origin = atrium_turret_top(idx);
    let start_angle = (idx as f32 + 0.5) / ATRIUM_TURRET_COUNT as f32 * TAU;
    let radius = rng.gen_range(PHASE4_RADIUS_MIN, PHASE4_RADIUS_MAX);

    let vy = rng.gen_range(PHASE4_LAUNCH_VY_MIN, PHASE4_LAUNCH_VY_MAX);
    let drop_y = (origin.y - (LEVEL_Y[3] + radius)).max(0.1);
    let t_flight = (vy + (vy * vy + 2.0 * PHASE4_GRAVITY * drop_y).sqrt()) / PHASE4_GRAVITY;

    let sweep_rad = rng.gen_range(PHASE4_ARC_DEG_MIN, PHASE4_ARC_DEG_MAX).to_radians();
    let end_angle = start_angle + sweep_rad * sweep_dir.signum();
    let end_r = rng.gen_range(PHASE4_LAND_RADIUS_MIN, PHASE4_LAND_RADIUS_MAX);
    let dest_xz = Vec2::new(end_r * end_angle.sin(), -end_r * end_angle.cos());
    let origin_xz = Vec2::new(origin.x, origin.z);
    let v_xz = (dest_xz - origin_xz) / t_flight;

    let entity = spawn_sphere(commands, meshes, material, origin, radius, None);
    commands.entity(entity).insert((
        SuperFlicksEntity,
        SuperFlicksTarget {
            phase: 3,
            hit_offset_y: 0.0,
        },
        ReactionTimer::new(),
        BallisticOrb {
            velocity: Vec3::new(v_xz.x, vy, v_xz.y),
            radius,
        },
    ));
}

/// Integrates parabolic projectile motion for Level 4 turret orbs and despawns them
/// silently (no miss penalty) upon touching the atrium floor.
pub fn update_phase4_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut orbs: Query<(Entity, &mut Transform, &mut BallisticOrb)>,
) {
    let dt = time.delta_secs();
    let floor_y = LEVEL_Y[3];

    for (entity, mut transform, mut orb) in &mut orbs {
        orb.velocity.y -= PHASE4_GRAVITY * dt;
        transform.translation += orb.velocity * dt;

        // Keep horizontal travel cleanly inside the octagonal atrium walls & clear of platform
        let xz = Vec2::new(transform.translation.x, transform.translation.z);
        let r = xz.length();
        if r > 0.01 {
            let radial = xz / r;
            let v_xz = Vec2::new(orb.velocity.x, orb.velocity.z);
            let radial_speed = v_xz.dot(radial);
            if r > 14.8 && radial_speed > 0.0 {
                let reflected = v_xz - 2.0 * radial_speed * radial;
                orb.velocity.x = reflected.x;
                orb.velocity.z = reflected.y;
                transform.translation.x = radial.x * 14.8;
                transform.translation.z = radial.y * 14.8;
            } else if r < 5.5 && radial_speed < 0.0 {
                let reflected = v_xz - 2.0 * radial_speed * radial;
                orb.velocity.x = reflected.x;
                orb.velocity.z = reflected.y;
                transform.translation.x = radial.x * 5.5;
                transform.translation.z = radial.y * 5.5;
            }
        }

        // Disappear silently on touching the teal floor (only failed shots are punished)
        if orb.velocity.y < 0.0 && transform.translation.y <= floor_y + orb.radius {
            commands.entity(entity).despawn();
        }
    }
}

/// Continuously launches sweeping projectiles strictly from turrets in front of the player's view.
#[allow(clippy::too_many_arguments)]
pub fn tick_phase4_turrets(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut state: ResMut<SuperFlicksState>,
    mut rng: ResMut<SuperFlicksRng>,
    cameras: Query<&GlobalTransform, With<PlayerCamera>>,
    orbs: Query<Entity, With<BallisticOrb>>,
) {
    if state.active_phase != 3 || !state.phase_spawned {
        return;
    }

    let active = orbs.iter().count();
    if active >= PHASE4_MAX_TARGETS {
        return;
    }

    if active == 0 && state.phase4_launch_timer > 0.12 {
        state.phase4_launch_timer = 0.12;
    }

    state.phase4_launch_timer -= time.delta_secs();
    if state.phase4_launch_timer <= 0.0 {
        let look_yaw = cameras
            .single()
            .map(|gt| {
                let fwd = *gt.forward();
                fwd.x.atan2(-fwd.z)
            })
            .unwrap_or(0.0);

        let (turret, sweep_dir) = pick_forward_turret(&mut rng, look_yaw, state.last_turret);
        spawn_turret_projectile(
            &mut commands,
            &mut meshes,
            &material,
            &mut rng,
            turret,
            sweep_dir,
        );
        state.last_turret = Some(turret);
        state.phase4_launch_timer =
            rng.gen_range(PHASE4_LAUNCH_DELAY_MIN, PHASE4_LAUNCH_DELAY_MAX);
    }
}

/// Picks a Level 4 floor turret directly in front of `look_yaw` (`|delta| <= 72 deg`)
/// and returns `(turret_index, sweep_dir)` aimed across the player's field of view.
fn pick_forward_turret(
    rng: &mut SuperFlicksRng,
    look_yaw: f32,
    last_turret: Option<usize>,
) -> (usize, f32) {
    let max_fov_half = 72.0f32.to_radians();
    let mut candidates: Vec<(usize, f32)> = Vec::with_capacity(6);

    for idx in 0..ATRIUM_TURRET_COUNT {
        let turret_angle = (idx as f32 + 0.5) / ATRIUM_TURRET_COUNT as f32 * TAU;
        let delta = wrap_pi(turret_angle - look_yaw);
        if delta.abs() <= max_fov_half {
            // Left turret (delta < 0) launches left->right (+1); right turret launches right->left (-1)
            let sweep_dir = if delta < 0.0 { 1.0 } else { -1.0 };
            candidates.push((idx, sweep_dir));
        }
    }

    if candidates.is_empty() {
        return (0, -1.0);
    }

    let filtered: Vec<(usize, f32)> = candidates
        .iter()
        .copied()
        .filter(|(idx, _)| Some(*idx) != last_turret)
        .collect();
    let pool = if filtered.is_empty() {
        &candidates
    } else {
        &filtered
    };
    pool[rng.gen_usize(pool.len())]
}

fn wrap_pi(mut angle: f32) -> f32 {
    while angle > PI {
        angle -= TAU;
    }
    while angle < -PI {
        angle += TAU;
    }
    angle
}
