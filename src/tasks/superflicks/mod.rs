//! SuperFlicks: 4-stage vertical elevator tower gauntlet grouping 4 exercises in one run:
//! - Phase 1 (Level 1): 5 fast-strafing 3D capsule bots (1 on every line: curb + 4 ledges)
//! - Phase 2 (Level 2): 6 fast, wide-range sweeping medium sphere orbs
//! - Phase 3 (Level 3): 12 well-separated micro precision static orbs
//! - Phase 4 (Level 4): Up to 5 ballistic projectile orbs launched from the 12 floor turrets

pub mod movement;
pub mod spawning;
pub mod turrets;
pub mod types;

use bevy::prelude::*;

use crate::core::camera::{PLAYER_POS, PlayerCamera};
use crate::core::hit::TargetHit;
use crate::core::input::PlayerLook;
use crate::core::round::RoundTimer;
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, TaskInfo, task_playing, task_selected};
use crate::props::targets::TargetMaterial;
use crate::world::superflicks_arena::{LEVEL_Y, SuperFlicksElevator};

use movement::{
    maintain_phase1_lines, move_phase1_bots, move_phase2_orbs, spawn_pop_particles,
    update_pop_particles,
};
use spawning::{spawn_phase_targets, spawn_replacement_target};
use turrets::{tick_phase4_turrets, update_phase4_projectiles};
use types::*;

pub const INFO: TaskInfo = TaskInfo {
    id: "superflicks",
    name: "SuperFlicks",
    description: "4-stage tower gauntlet: ledge bots, fast orbs, micro 360 & turret projectiles",
    scoring: ScoringMode::Click,
    is_sniper_mode: false,
};

pub fn plugin() -> impl Plugin {
    SuperFlicksPlugin
}

struct SuperFlicksPlugin;

impl Plugin for SuperFlicksPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SuperFlicksState>()
            .init_resource::<SuperFlicksRng>()
            .add_systems(
                OnEnter(AppState::Countdown),
                setup.run_if(task_selected(INFO.id)),
            )
            .add_systems(
                Update,
                (
                    advance_phase_and_elevator,
                    maintain_phase1_lines,
                    move_phase1_bots,
                    move_phase2_orbs,
                    update_phase4_projectiles,
                    tick_phase4_turrets,
                    handle_hits,
                    update_pop_particles,
                )
                    .chain()
                    .run_if(task_playing(INFO.id)),
            )
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

#[allow(clippy::too_many_arguments)]
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut state: ResMut<SuperFlicksState>,
    mut rng: ResMut<SuperFlicksRng>,
    mut look: ResMut<PlayerLook>,
    mut cameras: Query<&mut Transform, With<PlayerCamera>>,
    existing: Query<Entity, With<SuperFlicksEntity>>,
) {
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    *state = SuperFlicksState {
        active_phase: 0,
        elevator_from_y: LEVEL_Y[0],
        elevator_to_y: LEVEL_Y[0],
        elevator_elapsed: 0.0,
        elevator_moving: false,
        phase_spawned: true,
        current_y: LEVEL_Y[0],
        phase1_last_pos: [None; PHASE1_TARGET_COUNT],
        phase4_launch_timer: 0.0,
        last_turret: None,
    };

    *look = PlayerLook::default();
    if let Ok(mut cam_t) = cameras.single_mut() {
        cam_t.translation = Vec3::new(0.0, LEVEL_Y[0] + PLAYER_POS.y, 0.0);
        cam_t.rotation = Quat::IDENTITY;
    }

    spawn_phase_targets(0, &mut commands, &mut meshes, &material, &mut state, &mut rng);
}

#[allow(clippy::too_many_arguments)]
fn advance_phase_and_elevator(
    mut commands: Commands,
    time: Res<Time>,
    timer: Res<RoundTimer>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut state: ResMut<SuperFlicksState>,
    mut rng: ResMut<SuperFlicksRng>,
    targets: Query<Entity, With<SuperFlicksTarget>>,
    mut elevators: Query<&mut Transform, (With<SuperFlicksElevator>, Without<PlayerCamera>)>,
    mut cameras: Query<&mut Transform, (With<PlayerCamera>, Without<SuperFlicksElevator>)>,
) {
    let progress = if timer.duration > 0.0 {
        ((timer.duration - timer.remaining) / timer.duration).clamp(0.0, 0.9999)
    } else {
        0.0
    };
    let desired_phase = ((progress * 4.0).floor() as usize).min(LEVEL_Y.len() - 1);

    if desired_phase > state.active_phase {
        for entity in &targets {
            commands.entity(entity).despawn();
        }
        state.active_phase = desired_phase;
        state.elevator_from_y = state.current_y;
        state.elevator_to_y = LEVEL_Y[desired_phase];
        state.elevator_elapsed = 0.0;
        state.elevator_moving = true;
        state.phase_spawned = false;
    }

    if !state.elevator_moving {
        return;
    }

    state.elevator_elapsed += time.delta_secs();
    let t = (state.elevator_elapsed / ELEVATOR_DURATION_SECS).clamp(0.0, 1.0);
    let ease = t * t * (3.0 - 2.0 * t);
    state.current_y = state.elevator_from_y + (state.elevator_to_y - state.elevator_from_y) * ease;

    if let Ok(mut elev_t) = elevators.single_mut() {
        elev_t.translation.y = state.current_y;
    }
    if let Ok(mut cam_t) = cameras.single_mut() {
        cam_t.translation.y = state.current_y + PLAYER_POS.y;
    }

    if t >= SPAWN_AT_ELEVATOR_PROGRESS && !state.phase_spawned {
        let phase = state.active_phase;
        spawn_phase_targets(
            phase,
            &mut commands,
            &mut meshes,
            &material,
            &mut state,
            &mut rng,
        );
        state.phase_spawned = true;
    }

    if t >= 1.0 {
        state.current_y = state.elevator_to_y;
        state.elevator_moving = false;
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_hits(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut state: ResMut<SuperFlicksState>,
    mut rng: ResMut<SuperFlicksRng>,
    mut hits: MessageReader<TargetHit>,
    parents: Query<&ChildOf>,
    targets: Query<(Entity, &Transform, &SuperFlicksTarget)>,
) {
    let mut handled_roots: Vec<Entity> = Vec::new();

    for hit in hits.read() {
        let root = parents
            .get(hit.entity)
            .map(|p| p.parent())
            .unwrap_or(hit.entity);

        if handled_roots.contains(&root) {
            continue;
        }
        let Ok((_, transform, target)) = targets.get(root) else {
            continue;
        };
        handled_roots.push(root);

        // Phase 1 capsule bots (hit_zone.is_some()) are despawned by core::hit::consume_shots
        // and instantly replenished on their line by `maintain_phase1_lines`.
        if hit.hit_zone.is_some() || target.phase == 0 {
            continue;
        }

        let hit_pos = transform.translation;
        let burst_pos = hit_pos + Vec3::Y * target.hit_offset_y;
        spawn_pop_particles(&mut commands, &mut meshes, &material, burst_pos, &mut rng);
        commands.entity(root).despawn();

        if !state.phase_spawned || target.phase != state.active_phase {
            continue;
        }

        // Phase 4 projectiles are replenished by the floor turrets; gently nudge timer on hit
        if state.active_phase == 3 {
            state.phase4_launch_timer = state.phase4_launch_timer.min(0.20);
            continue;
        }

        let occupied: Vec<Vec3> = targets
            .iter()
            .filter(|(e, _, t)| t.phase == state.active_phase && !handled_roots.contains(e))
            .map(|(_, tr, _)| tr.translation)
            .collect();

        spawn_replacement_target(
            state.active_phase,
            &mut commands,
            &mut meshes,
            &material,
            &mut rng,
            &occupied,
            hit_pos,
        );
    }
}

fn cleanup(
    mut commands: Commands,
    entities: Query<Entity, With<SuperFlicksEntity>>,
    mut state: ResMut<SuperFlicksState>,
    mut cameras: Query<&mut Transform, With<PlayerCamera>>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    *state = SuperFlicksState::default();
    if let Ok(mut cam_t) = cameras.single_mut() {
        cam_t.translation = PLAYER_POS;
    }
}
