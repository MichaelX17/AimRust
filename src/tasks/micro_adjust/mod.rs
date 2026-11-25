//! Micro Adjust precision task:
//! Modeled after Micro_Adjust.mp4 on the Default arena with a compact far backboard wall:
//! - Up to 2 active small 3D glossy spheres hovering just in front of the backboard
//! - Tight micro-adjustment clustering (~75% close clusters, ~25% wider wall repositions)
//! - Crisp ~0.20s respawn delay after each hit before the replacement target appears
//! - Follows Settings target color customization and uses the standard AimRust HUD/UI

pub mod types;

use bevy::prelude::*;

use crate::core::camera::PlayerCamera;
use crate::core::config::{ColorsCfg, Settings};
use crate::core::hit::{Miss, Target, TargetHit, TargetShape};
use crate::core::hit_zone::ReactionTimer;
use crate::core::input::PlayerLook;
use crate::core::score::RoundScore;
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, TaskInfo, task_playing, task_selected};
use types::*;

pub const INFO: TaskInfo = TaskInfo {
    id: "micro_adjust",
    name: "Micro Adjust",
    description: "High-precision micro-adjustments against small 3D targets",
    scoring: ScoringMode::Click,
    is_sniper_mode: false,
};

pub fn plugin() -> impl Plugin {
    MicroAdjustPlugin
}

struct MicroAdjustPlugin;

impl Plugin for MicroAdjustPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MicroAdjustStats>()
            .init_resource::<MicroAdjustRng>()
            .add_systems(
                OnEnter(AppState::Countdown),
                setup.run_if(task_selected(INFO.id)),
            )
            .add_systems(
                Update,
                (
                    sync_target_material,
                    update_pop_particles,
                    handle_hits,
                    process_pending_respawns,
                    handle_misses,
                )
                    .chain()
                    .run_if(task_playing(INFO.id)),
            )
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

fn pick_target_coords(existing: &[Vec2], rng: &mut MicroAdjustRng) -> Vec2 {
    if existing.is_empty() {
        return Vec2::new(
            rng.gen_range(X_MIN * 0.75, X_MAX * 0.75),
            rng.gen_range(Y_MIN + 0.25, Y_MAX - 0.25),
        );
    }

    // ~75% tight micro-adjustment near existing target, ~25% wider reposition across wall
    let max_sep = if rng.gen_f32() < 0.75 {
        MICRO_MAX_SEPARATION
    } else {
        WIDE_MAX_SEPARATION
    };

    let anchor = existing[0];
    let mut best = Vec2::new(0.0, 1.7);
    let mut best_score = -1.0f32;

    for _ in 0..60 {
        let angle = rng.gen_range(0.0, std::f32::consts::TAU);
        let dist = rng.gen_range(MIN_SEPARATION, max_sep);
        let cand = Vec2::new(
            (anchor.x + angle.cos() * dist).clamp(X_MIN, X_MAX),
            (anchor.y + angle.sin() * dist).clamp(Y_MIN, Y_MAX),
        );

        let min_dist = existing
            .iter()
            .map(|other| cand.distance(*other))
            .fold(f32::INFINITY, f32::min);

        if min_dist >= MIN_SEPARATION && min_dist <= max_sep {
            return cand;
        }

        let score = if min_dist < MIN_SEPARATION {
            min_dist / MIN_SEPARATION
        } else {
            (10.0 - min_dist).max(0.0)
        };
        if score > best_score {
            best_score = score;
            best = cand;
        }
    }

    best
}

fn spawn_single_target(
    commands: &mut Commands,
    assets: &MicroAdjustAssets,
    coords: Vec2,
) -> Entity {
    let target_pos = Vec3::new(coords.x, coords.y, TARGET_Z);

    commands
        .spawn((
            Mesh3d(assets.sphere_mesh.clone()),
            MeshMaterial3d(assets.target_material.clone()),
            Transform::from_translation(target_pos),
            Target {
                shape: TargetShape::Sphere {
                    radius: TARGET_RADIUS,
                },
                lifetime: None,
            },
            MicroAdjustTarget,
            ReactionTimer::new(),
            MicroAdjustEntity,
        ))
        .id()
}

fn spawn_pop_particles(
    commands: &mut Commands,
    assets: &MicroAdjustAssets,
    center: Vec3,
    rng: &mut MicroAdjustRng,
) {
    for _ in 0..6 {
        let vx = rng.gen_range(-1.4, 1.4);
        let vy = rng.gen_range(-0.9, 1.6);
        let vz = rng.gen_range(0.2, 1.5);

        commands.spawn((
            Mesh3d(assets.particle_mesh.clone()),
            MeshMaterial3d(assets.particle_material.clone()),
            Transform::from_translation(center),
            MicroAdjustPopParticle {
                velocity: Vec3::new(vx, vy, vz),
                timer: Timer::from_seconds(0.14, TimerMode::Once),
            },
            MicroAdjustEntity,
        ));
    }
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

    let base_shadow_mat = materials.add(StandardMaterial {
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
        MicroAdjustEntity,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(WALL_WIDTH + 0.4, 0.002, 0.4))),
        MeshMaterial3d(base_shadow_mat),
        Transform::from_xyz(0.0, 0.0018, WALL_Z),
        MicroAdjustEntity,
    ));
}

#[allow(clippy::too_many_arguments)]
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut stats: ResMut<MicroAdjustStats>,
    mut rng: ResMut<MicroAdjustRng>,
    mut look: ResMut<PlayerLook>,
    mut cameras: Query<&mut Transform, With<PlayerCamera>>,
    settings: Option<Res<Settings>>,
) {
    *stats = MicroAdjustStats::default();

    // Center camera down the white alignment line toward the backboard
    *look = PlayerLook::default();
    if let Ok(mut cam_t) = cameras.single_mut() {
        cam_t.translation = Vec3::new(0.0, 1.7, 0.0);
        cam_t.rotation = Quat::IDENTITY;
    }

    spawn_backboard_wall(&mut commands, &mut meshes, &mut materials);

    let [r, g, b] = settings
        .as_ref()
        .map(|s| s.colors.target)
        .unwrap_or(ColorsCfg::default().target);
    let target_color = Color::srgb_u8(r, g, b);

    // 3D glossy PBR finish matching Peek Defense targets while honoring Settings target color
    let target_material = materials.add(StandardMaterial {
        base_color: target_color,
        perceptual_roughness: 0.20,
        metallic: 0.25,
        reflectance: 0.85,
        ..default()
    });

    let particle_material = materials.add(StandardMaterial {
        base_color: target_color,
        perceptual_roughness: 0.35,
        metallic: 0.15,
        ..default()
    });

    let assets = MicroAdjustAssets {
        sphere_mesh: meshes.add(Sphere::new(TARGET_RADIUS).mesh().uv(32, 18)),
        particle_mesh: meshes.add(Sphere::new(0.02)),
        target_material,
        particle_material,
    };

    let mut coords_list: Vec<Vec2> = Vec::with_capacity(ACTIVE_TARGET_COUNT);
    for _ in 0..ACTIVE_TARGET_COUNT {
        let coords = pick_target_coords(&coords_list, &mut rng);
        coords_list.push(coords);
        spawn_single_target(&mut commands, &assets, coords);
    }

    commands.insert_resource(assets);
}

fn sync_target_material(
    settings: Option<Res<Settings>>,
    assets: Option<Res<MicroAdjustAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (Some(settings), Some(assets)) = (settings, assets) else {
        return;
    };
    if !settings.is_changed() {
        return;
    }

    let [r, g, b] = settings.colors.target;
    let desired = Color::srgb_u8(r, g, b);

    if let Some(mut mat) = materials.get_mut(&assets.target_material) {
        mat.base_color = desired;
    }
    if let Some(mut mat) = materials.get_mut(&assets.particle_material) {
        mat.base_color = desired;
    }
}

fn update_pop_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut MicroAdjustPopParticle)>,
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

#[allow(clippy::too_many_arguments)]
fn handle_hits(
    mut commands: Commands,
    assets: Res<MicroAdjustAssets>,
    mut hits: MessageReader<TargetHit>,
    mut score: ResMut<RoundScore>,
    mut stats: ResMut<MicroAdjustStats>,
    mut rng: ResMut<MicroAdjustRng>,
    targets: Query<(Entity, &Transform, &ReactionTimer), With<MicroAdjustTarget>>,
) {
    for hit in hits.read() {
        let Ok((entity, transform, reaction)) = targets.get(hit.entity) else {
            continue;
        };

        spawn_pop_particles(&mut commands, &assets, transform.translation, &mut rng);
        commands.entity(entity).despawn();

        stats.streak += 1;
        let streak_bonus = stats.streak.saturating_sub(1).min(10) as i32;
        score.points += 10 + streak_bonus;
        score.hits += 1;
        stats.targets_destroyed += 1;
        stats.total_ttk_ms += reaction.elapsed_ms;

        // Schedule delayed replacement (~0.20s) matching Micro_Adjust.mp4 cadence
        stats
            .pending_respawns
            .push(Timer::from_seconds(RESPAWN_DELAY_SECS, TimerMode::Once));
    }
}

fn process_pending_respawns(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<MicroAdjustAssets>,
    mut stats: ResMut<MicroAdjustStats>,
    mut rng: ResMut<MicroAdjustRng>,
    targets: Query<&Transform, With<MicroAdjustTarget>>,
) {
    if stats.pending_respawns.is_empty() {
        return;
    }

    let mut ready_count = 0usize;
    for timer in &mut stats.pending_respawns {
        timer.tick(time.delta());
        if timer.is_finished() {
            ready_count += 1;
        }
    }
    stats.pending_respawns.retain(|t| !t.is_finished());

    if ready_count == 0 {
        return;
    }

    let mut existing_coords: Vec<Vec2> = targets
        .iter()
        .map(|t| Vec2::new(t.translation.x, t.translation.y))
        .collect();

    for _ in 0..ready_count {
        if existing_coords.len() >= ACTIVE_TARGET_COUNT {
            break;
        }
        let coords = pick_target_coords(&existing_coords, &mut rng);
        existing_coords.push(coords);
        spawn_single_target(&mut commands, &assets, coords);
    }
}

fn handle_misses(
    mut misses: MessageReader<Miss>,
    mut score: ResMut<RoundScore>,
    mut stats: ResMut<MicroAdjustStats>,
) {
    for _ in misses.read() {
        stats.streak = 0;
        score.points = (score.points - 3).max(0);
        score.misses += 1;
    }
}

fn cleanup(
    mut commands: Commands,
    query: Query<Entity, With<MicroAdjustEntity>>,
    mut stats: ResMut<MicroAdjustStats>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    *stats = MicroAdjustStats::default();
}
