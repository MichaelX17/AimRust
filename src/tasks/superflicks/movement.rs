//! Movement systems for Phase 1 fast ledge-strafing bots (with infinite per-line respawn),
//! Phase 2 wide-range sweeping orbs, and hit pop particles.

use bevy::prelude::*;

use crate::props::targets::TargetMaterial;
use crate::world::superflicks_arena::{LEVEL1_LEDGE_RADIUS, LEVEL1_LEDGE_YS};

use super::spawning::{cylindrical_pos, spawn_phase1_bot};
use super::types::*;

pub fn move_phase1_bots(
    time: Res<Time>,
    mut state: ResMut<SuperFlicksState>,
    mut rng: ResMut<SuperFlicksRng>,
    mut bots: Query<(&mut Transform, &mut LedgeStrafe)>,
) {
    if state.active_phase != 0 || state.elevator_moving {
        return;
    }

    let dt = time.delta_secs();
    for (mut transform, mut strafe) in &mut bots {
        strafe.change_timer -= dt;
        if strafe.change_timer <= 0.0 {
            let speed = rng.gen_range(PHASE1_ANGULAR_SPEED_MIN, PHASE1_ANGULAR_SPEED_MAX);
            strafe.angular_vel = speed * rng.gen_sign();
            strafe.change_timer = rng.gen_range(PHASE1_STRAFE_HOLD_MIN, PHASE1_STRAFE_HOLD_MAX);
        }

        strafe.angle += strafe.angular_vel * dt;
        if strafe.angle < -strafe.half_arc {
            strafe.angle = -strafe.half_arc;
            strafe.angular_vel = strafe.angular_vel.abs();
        } else if strafe.angle > strafe.half_arc {
            strafe.angle = strafe.half_arc;
            strafe.angular_vel = -strafe.angular_vel.abs();
        }

        let tier = strafe.tier.min(LEVEL1_LEDGE_YS.len() - 1);
        let ledge_y = LEVEL1_LEDGE_YS[tier];
        let pos = cylindrical_pos(LEVEL1_LEDGE_RADIUS, strafe.angle, ledge_y);
        *transform =
            Transform::from_translation(pos).looking_at(Vec3::new(0.0, ledge_y, 0.0), Vec3::Y);
        state.phase1_last_pos[tier] = Some(pos);
    }
}

/// Guarantees infinite, instant respawns on every Level 1 line: whenever `core::hit` despawns
/// a headshotted/bodyshotted capsule bot, its line immediately bursts particles and spawns a new bot.
pub fn maintain_phase1_lines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut state: ResMut<SuperFlicksState>,
    mut rng: ResMut<SuperFlicksRng>,
    bots: Query<(&Transform, &LedgeStrafe)>,
) {
    if state.active_phase != 0 || !state.phase_spawned || state.elevator_moving {
        return;
    }

    let mut present = [false; PHASE1_TARGET_COUNT];
    let mut occupied: Vec<Vec3> = Vec::with_capacity(PHASE1_TARGET_COUNT + 1);
    for (tr, strafe) in &bots {
        if strafe.tier < PHASE1_TARGET_COUNT {
            present[strafe.tier] = true;
        }
        occupied.push(tr.translation);
    }

    for (tier, &has_bot) in present.iter().enumerate() {
        if has_bot {
            continue;
        }

        let mut avoid = occupied.clone();
        if let Some(vacated) = state.phase1_last_pos[tier] {
            spawn_pop_particles(
                &mut commands,
                &mut meshes,
                &material,
                vacated + Vec3::Y * 0.95,
                &mut rng,
            );
            avoid.push(vacated);
        }

        let new_pos = spawn_phase1_bot(&mut commands, &mut meshes, &material, &mut rng, tier, &avoid);
        state.phase1_last_pos[tier] = Some(new_pos);
        occupied.push(new_pos);
    }
}

pub fn move_phase2_orbs(
    time: Res<Time>,
    mut rng: ResMut<SuperFlicksRng>,
    mut orbs: Query<(&mut Transform, &mut OrbSweep)>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut sweep) in &mut orbs {
        sweep.change_timer -= dt;
        if sweep.change_timer <= 0.0 {
            sweep.angular_vel =
                rng.gen_range(PHASE2_ANGULAR_SPEED_MIN, PHASE2_ANGULAR_SPEED_MAX) * rng.gen_sign();
            sweep.vertical_vel = rng.gen_range(PHASE2_VERTICAL_SPEED_MIN, PHASE2_VERTICAL_SPEED_MAX)
                * rng.gen_sign();
            sweep.change_timer = rng.gen_range(PHASE2_HOLD_MIN, PHASE2_HOLD_MAX);
        }

        sweep.angle += sweep.angular_vel * dt;
        if sweep.angle < -sweep.yaw_limit {
            sweep.angle = -sweep.yaw_limit;
            sweep.angular_vel = sweep.angular_vel.abs();
        } else if sweep.angle > sweep.yaw_limit {
            sweep.angle = sweep.yaw_limit;
            sweep.angular_vel = -sweep.angular_vel.abs();
        }

        sweep.y += sweep.vertical_vel * dt;
        if sweep.y < sweep.y_min {
            sweep.y = sweep.y_min;
            sweep.vertical_vel = sweep.vertical_vel.abs();
        } else if sweep.y > sweep.y_max {
            sweep.y = sweep.y_max;
            sweep.vertical_vel = -sweep.vertical_vel.abs();
        }

        transform.translation = cylindrical_pos(sweep.radius, sweep.angle, sweep.y);
    }
}

/// Spawns 6 small pop particles tinted by `TargetMaterial` at a destroyed target's center.
pub fn spawn_pop_particles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    pos: Vec3,
    rng: &mut SuperFlicksRng,
) {
    let particle_mesh = meshes.add(Sphere::new(0.032));
    for _ in 0..6 {
        let vx = rng.gen_range(-2.1, 2.1);
        let vy = rng.gen_range(-1.2, 2.3);
        let vz = rng.gen_range(-2.1, 2.1);

        commands.spawn((
            Mesh3d(particle_mesh.clone()),
            MeshMaterial3d(material.0.clone()),
            Transform::from_translation(pos),
            SuperFlicksPopParticle {
                velocity: Vec3::new(vx, vy, vz),
                timer: Timer::from_seconds(0.14, TimerMode::Once),
            },
            SuperFlicksEntity,
        ));
    }
}

pub fn update_pop_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut SuperFlicksPopParticle)>,
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
