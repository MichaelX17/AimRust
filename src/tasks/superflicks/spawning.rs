//! Target spawners for Phases 1..=3 of the SuperFlicks tower gauntlet.

use bevy::prelude::*;

use crate::core::hit_zone::ReactionTimer;
use crate::props::targets::{TargetMaterial, spawn_capsule_target, spawn_sphere};
use crate::world::superflicks_arena::{LEVEL_Y, LEVEL1_LEDGE_RADIUS, LEVEL1_LEDGE_YS};

use super::types::*;

#[derive(Clone, Copy)]
struct OrbSpec {
    floor_y: f32,
    dist_min: f32,
    dist_max: f32,
    y_min: f32,
    y_max: f32,
    yaw_half: f32,
    min_sep: f32,
    radius: f32,
}

fn orb_spec(phase: usize) -> OrbSpec {
    match phase {
        1 => OrbSpec {
            floor_y: LEVEL_Y[1],
            dist_min: PHASE2_DIST_MIN,
            dist_max: PHASE2_DIST_MAX,
            y_min: PHASE2_Y_MIN,
            y_max: PHASE2_Y_MAX,
            yaw_half: PHASE2_YAW_HALF_DEG.to_radians(),
            min_sep: PHASE2_MIN_SEP,
            radius: PHASE2_RADIUS,
        },
        _ => OrbSpec {
            floor_y: LEVEL_Y[2],
            dist_min: PHASE3_DIST_MIN,
            dist_max: PHASE3_DIST_MAX,
            y_min: PHASE3_Y_MIN,
            y_max: PHASE3_Y_MAX,
            yaw_half: PHASE3_YAW_HALF_DEG.to_radians(),
            min_sep: PHASE3_MIN_SEP,
            radius: PHASE3_RADIUS,
        },
    }
}

/// Spawns the initial target wave for the specified phase (`0..=3`).
pub fn spawn_phase_targets(
    phase: usize,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    state: &mut SuperFlicksState,
    rng: &mut SuperFlicksRng,
) {
    match phase {
        0 => {
            // Strictly 1 strafing bot on EVERY one of the 5 Level 1 lines
            let mut occupied = Vec::with_capacity(PHASE1_TARGET_COUNT);
            for tier in 0..PHASE1_TARGET_COUNT {
                let pos = spawn_phase1_bot(commands, meshes, material, rng, tier, &occupied);
                state.phase1_last_pos[tier] = Some(pos);
                occupied.push(pos);
            }
        }
        1 | 2 => {
            let count = if phase == 1 { PHASE2_TARGET_COUNT } else { PHASE3_TARGET_COUNT };
            let mut occupied = Vec::with_capacity(count);
            for _ in 0..count {
                let pos = spawn_phase2_or_3_orb(phase, commands, meshes, material, rng, &occupied);
                occupied.push(pos);
            }
        }
        _ => {
            // Let `tick_phase4_turrets` immediately fire from turrets in front of player's live view
            state.phase4_launch_timer = 0.0;
        }
    }
}

/// Spawns a single replacement target for Phases 2..=3.
pub fn spawn_replacement_target(
    phase: usize,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    rng: &mut SuperFlicksRng,
    occupied: &[Vec3],
    vacated: Vec3,
) {
    let mut avoid = Vec::with_capacity(occupied.len() + 1);
    avoid.extend_from_slice(occupied);
    avoid.push(vacated);

    if phase == 1 || phase == 2 {
        spawn_phase2_or_3_orb(phase, commands, meshes, material, rng, &avoid);
    }
}

/// Spawns a Phase 1 fast strafing capsule bot on `LEVEL1_LEDGE_YS[tier]`.
pub fn spawn_phase1_bot(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    rng: &mut SuperFlicksRng,
    tier: usize,
    avoid: &[Vec3],
) -> Vec3 {
    let tier_idx = tier.min(LEVEL1_LEDGE_YS.len() - 1);
    let ledge_y = LEVEL1_LEDGE_YS[tier_idx];
    let half_arc = PHASE1_LEDGE_HALF_DEGS[tier_idx].to_radians();

    let mut best_angle = rng.gen_range(-half_arc, half_arc);
    let mut best_sep = -1.0f32;

    for _ in 0..48 {
        let cand_angle = rng.gen_range(-half_arc, half_arc);
        let cand_pos = cylindrical_pos(LEVEL1_LEDGE_RADIUS, cand_angle, ledge_y);
        let min_dist = min_distance(cand_pos, avoid);
        if min_dist >= LEVEL1_LEDGE_RADIUS * PHASE1_MIN_ANGLE_SEP {
            best_angle = cand_angle;
            break;
        }
        if min_dist > best_sep {
            best_sep = min_dist;
            best_angle = cand_angle;
        }
    }

    let pos = cylindrical_pos(LEVEL1_LEDGE_RADIUS, best_angle, ledge_y);
    let speed = rng.gen_range(PHASE1_ANGULAR_SPEED_MIN, PHASE1_ANGULAR_SPEED_MAX) * rng.gen_sign();
    let hold = rng.gen_range(PHASE1_STRAFE_HOLD_MIN, PHASE1_STRAFE_HOLD_MAX);

    let root = spawn_capsule_target(commands, meshes, material.0.clone(), pos);
    commands.entity(root).insert((
        Transform::from_translation(pos).looking_at(Vec3::new(0.0, ledge_y, 0.0), Vec3::Y),
        SuperFlicksEntity,
        SuperFlicksTarget {
            phase: 0,
            hit_offset_y: 0.95,
        },
        LedgeStrafe {
            tier: tier_idx,
            angle: best_angle,
            angular_vel: speed,
            half_arc,
            change_timer: hold,
        },
    ));
    pos
}

/// Spawns one orb target for Phase 2 (fast sweeping) or Phase 3 (well-spaced static micro).
fn spawn_phase2_or_3_orb(
    phase: usize,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    rng: &mut SuperFlicksRng,
    avoid: &[Vec3],
) -> Vec3 {
    let spec = orb_spec(phase);
    let (dist, angle, y) = sample_orb_coords(rng, spec, avoid);
    let pos = cylindrical_pos(dist, angle, y);

    let entity = spawn_sphere(commands, meshes, material, pos, spec.radius, None);
    let mut ecmd = commands.entity(entity);
    ecmd.insert((
        SuperFlicksEntity,
        SuperFlicksTarget {
            phase,
            hit_offset_y: 0.0,
        },
        ReactionTimer::new(),
    ));

    if phase == 1 {
        ecmd.insert(OrbSweep {
            radius: dist,
            angle,
            y,
            angular_vel: rng.gen_range(PHASE2_ANGULAR_SPEED_MIN, PHASE2_ANGULAR_SPEED_MAX)
                * rng.gen_sign(),
            vertical_vel: rng.gen_range(PHASE2_VERTICAL_SPEED_MIN, PHASE2_VERTICAL_SPEED_MAX)
                * rng.gen_sign(),
            yaw_limit: spec.yaw_half,
            y_min: spec.floor_y + spec.y_min,
            y_max: spec.floor_y + spec.y_max,
            change_timer: rng.gen_range(PHASE2_HOLD_MIN, PHASE2_HOLD_MAX),
        });
    }

    pos
}

fn sample_orb_coords(rng: &mut SuperFlicksRng, spec: OrbSpec, avoid: &[Vec3]) -> (f32, f32, f32) {
    let mut best = (
        rng.gen_range(spec.dist_min, spec.dist_max),
        rng.gen_range(-spec.yaw_half, spec.yaw_half),
        spec.floor_y + rng.gen_range(spec.y_min, spec.y_max),
    );
    let mut best_sep = -1.0f32;
    for _ in 0..80 {
        let d = rng.gen_range(spec.dist_min, spec.dist_max);
        let a = rng.gen_range(-spec.yaw_half, spec.yaw_half);
        let y = spec.floor_y + rng.gen_range(spec.y_min, spec.y_max);
        let sep = min_distance(cylindrical_pos(d, a, y), avoid);
        if sep >= spec.min_sep {
            return (d, a, y);
        }
        if sep > best_sep {
            best_sep = sep;
            best = (d, a, y);
        }
    }
    best
}

pub fn cylindrical_pos(radius: f32, angle: f32, y: f32) -> Vec3 {
    Vec3::new(radius * angle.sin(), y, -radius * angle.cos())
}

fn min_distance(candidate: Vec3, points: &[Vec3]) -> f32 {
    points
        .iter()
        .map(|p| candidate.distance(*p))
        .fold(f32::INFINITY, f32::min)
}
