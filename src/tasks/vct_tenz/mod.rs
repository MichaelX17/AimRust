//! VCT NA - TenZ: escalating corridor defense rush.
//! Targets spawn at the far neon portal and rush toward the player line at progressively
//! faster speeds and tighter cadence. If 3 targets pass the player line, the task ends.

pub mod spawning;
pub mod types;

use bevy::prelude::*;

use crate::core::camera::{PLAYER_POS, PlayerCamera};
use crate::core::hit::{Miss, TargetHit};
use crate::core::input::PlayerLook;
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, TaskInfo, task_playing, task_selected};
use crate::props::targets::TargetMaterial;
use crate::world::tenz_corridor_arena::{
    BREACH_Z, SPAWN_Z, TenZBreachGate, TenZIndicatorMaterials, TenZLifePip,
};

use spawning::{sample_portal_xy, spawn_orb, spawn_pop_particles, update_pop_particles};
use types::*;

pub const INFO: TaskInfo = TaskInfo {
    id: "vct_tenz",
    name: "VCT NA - TenZ",
    description: "Stop rushing targets down the neon corridor; 3 breaches end the run",
    scoring: ScoringMode::Click,
    is_sniper_mode: false,
};

pub fn plugin() -> impl Plugin {
    TenZPlugin
}

struct TenZPlugin;

impl Plugin for TenZPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TenZState>()
            .init_resource::<TenZRng>()
            .add_systems(
                OnEnter(AppState::Countdown),
                setup.run_if(task_selected(INFO.id)),
            )
            .add_systems(
                Update,
                (
                    spawn_rushing_targets,
                    advance_targets_and_check_breaches,
                    handle_hits,
                    update_pop_particles,
                    update_gate_flash,
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
    mut state: ResMut<TenZState>,
    mut rng: ResMut<TenZRng>,
    mut look: ResMut<PlayerLook>,
    mut cameras: Query<&mut Transform, With<PlayerCamera>>,
    existing: Query<Entity, With<TenZEntity>>,
) {
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    *state = TenZState {
        elapsed: 0.0,
        spawn_timer: 0.45,
        breaches: 0,
        gate_flash_timer: 0.0,
        last_spawn_xy: None,
    };

    *look = PlayerLook::default();
    if let Ok(mut cam_t) = cameras.single_mut() {
        cam_t.translation = PLAYER_POS;
        cam_t.rotation = Quat::IDENTITY;
    }

    // Spawn an opening target at the far portal so the player locks on during countdown
    let xy = sample_portal_xy(&mut rng, &[]);
    state.last_spawn_xy = Some(xy);
    spawn_orb(&mut commands, &mut meshes, &material, xy, BASE_RUSH_SPEED);
}

#[allow(clippy::too_many_arguments)]
fn spawn_rushing_targets(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut state: ResMut<TenZState>,
    mut rng: ResMut<TenZRng>,
    targets: Query<&Transform, With<TenZTarget>>,
) {
    let dt = time.delta_secs();
    state.elapsed += dt;

    let active_count = targets.iter().count();
    if active_count >= MAX_ACTIVE_TARGETS {
        return;
    }

    // Fast-track spawn whenever the corridor is clear so tempo stays brisk
    if active_count == 0 && state.spawn_timer > 0.14 {
        state.spawn_timer = 0.14;
    }

    state.spawn_timer -= dt;
    if state.spawn_timer > 0.0 {
        return;
    }

    let ramp = (state.elapsed / RAMP_DURATION_SECS).clamp(0.0, 1.5);
    let speed = BASE_RUSH_SPEED + RAMP_RUSH_SPEED * ramp;
    let interval = (BASE_SPAWN_INTERVAL - 0.56 * ramp.min(1.0)).max(MIN_SPAWN_INTERVAL);

    let mut avoid: Vec<Vec2> = targets
        .iter()
        .filter(|tr| tr.translation.z < SPAWN_Z + 8.0)
        .map(|tr| Vec2::new(tr.translation.x, tr.translation.y))
        .collect();
    if let Some(last) = state.last_spawn_xy {
        avoid.push(last);
    }

    let xy = sample_portal_xy(&mut rng, &avoid);
    state.last_spawn_xy = Some(xy);
    spawn_orb(&mut commands, &mut meshes, &material, xy, speed);
    state.spawn_timer = interval * rng.gen_range(0.88, 1.12);
}

#[allow(clippy::too_many_arguments)]
fn advance_targets_and_check_breaches(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<TenZState>,
    indicators: Option<Res<TenZIndicatorMaterials>>,
    mut misses: MessageWriter<Miss>,
    mut next_state: ResMut<NextState<AppState>>,
    mut targets: Query<(Entity, &mut Transform, &TenZTarget)>,
    mut pips: Query<(&TenZLifePip, &mut MeshMaterial3d<StandardMaterial>), Without<TenZBreachGate>>,
    mut gates: Query<&mut MeshMaterial3d<StandardMaterial>, With<TenZBreachGate>>,
) {
    let dt = time.delta_secs();
    let mut breached_now = false;

    for (entity, mut transform, target) in &mut targets {
        transform.translation.z += target.speed * dt;

        if transform.translation.z >= BREACH_Z {
            commands.entity(entity).despawn();
            state.breaches += 1;
            breached_now = true;
            misses.write(Miss);
        }
    }

    if !breached_now {
        return;
    }

    state.gate_flash_timer = GATE_FLASH_SECS;

    if let Some(mats) = indicators {
        for (pip, mut mat) in &mut pips {
            if pip.0 < state.breaches {
                mat.0 = mats.breached.clone();
            }
        }
        for mut mat in &mut gates {
            mat.0 = mats.gate_flash.clone();
        }
    }

    // End the run immediately once 3 targets have passed the player line
    if state.breaches >= MAX_BREACHES {
        next_state.set(AppState::Results);
    }
}

fn handle_hits(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut rng: ResMut<TenZRng>,
    mut hits: MessageReader<TargetHit>,
    targets: Query<(Entity, &Transform), With<TenZTarget>>,
) {
    for hit in hits.read() {
        let Ok((entity, transform)) = targets.get(hit.entity) else {
            continue;
        };
        spawn_pop_particles(&mut commands, &mut meshes, &material, transform.translation, &mut rng);
        commands.entity(entity).despawn();
    }
}

fn update_gate_flash(
    time: Res<Time>,
    mut state: ResMut<TenZState>,
    indicators: Option<Res<TenZIndicatorMaterials>>,
    mut gates: Query<&mut MeshMaterial3d<StandardMaterial>, With<TenZBreachGate>>,
) {
    if state.gate_flash_timer <= 0.0 {
        return;
    }
    state.gate_flash_timer -= time.delta_secs();
    if state.gate_flash_timer <= 0.0
        && let Some(mats) = indicators
    {
        for mut mat in &mut gates {
            mat.0 = mats.gate_normal.clone();
        }
    }
}

fn cleanup(
    mut commands: Commands,
    entities: Query<Entity, With<TenZEntity>>,
    mut state: ResMut<TenZState>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    *state = TenZState::default();
}
