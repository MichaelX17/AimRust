//! Multishot Ultimate flicking task:
//! Fast-paced rapid target-switching scenario modeled after Aimlabs Multishot Ultimate:
//! up to 5 active 3D glossy spheres growing and shrinking in front of a backboard wall
//! on the Default arena, tinted by the user's Settings target color.

pub mod spawning;
pub mod types;

use bevy::prelude::*;

use crate::core::config::{ColorsCfg, Settings};
use crate::core::hit::{Miss, Target, TargetHit, TargetShape};
use crate::core::hit_zone::ReactionTimer;
use crate::core::score::RoundScore;
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, TaskInfo, task_playing, task_selected};
use spawning::*;
use types::*;

pub const INFO: TaskInfo = TaskInfo {
    id: "multishot_ultimate",
    name: "Multishot Ultimate",
    description: "Rapid target switching against dynamic shrinking targets",
    scoring: ScoringMode::Click,
    is_sniper_mode: false,
};

pub fn plugin() -> impl Plugin {
    MultishotUltimatePlugin
}

struct MultishotUltimatePlugin;

impl Plugin for MultishotUltimatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MultishotUltimateStats>()
            .init_resource::<UltimateRng>()
            .add_systems(
                OnEnter(AppState::Countdown),
                setup.run_if(task_selected(INFO.id)),
            )
            .add_systems(
                Update,
                (
                    sync_target_material,
                    update_shrinking_targets,
                    update_particles,
                    handle_hits,
                    handle_misses,
                )
                    .run_if(task_playing(INFO.id)),
            )
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

fn spawn_single_target(
    commands: &mut Commands,
    assets: &MultishotAssets,
    pos: Vec3,
    rng: &mut UltimateRng,
) -> Entity {
    let (max_radius, lifetime_sec) = sample_target_size(rng);

    commands
        .spawn((
            Mesh3d(assets.sphere_mesh.clone()),
            MeshMaterial3d(assets.target_material.clone()),
            Transform::from_translation(pos).with_scale(Vec3::splat(SPAWN_RADIUS)),
            Target {
                shape: TargetShape::Sphere {
                    radius: SPAWN_RADIUS,
                },
                lifetime: None,
            },
            MultishotTarget {
                lifetime: Timer::from_seconds(lifetime_sec, TimerMode::Once),
                max_radius,
                current_radius: SPAWN_RADIUS,
            },
            ReactionTimer::new(),
            MultishotUltimateEntity,
        ))
        .id()
}

fn spawn_hit_particles(
    commands: &mut Commands,
    assets: &MultishotAssets,
    center: Vec3,
    rng: &mut UltimateRng,
) {
    for _ in 0..8 {
        let vx = rng.gen_range(-2.8, 2.8);
        let vy = rng.gen_range(-1.6, 3.2);
        let vz = rng.gen_range(-1.5, 2.8);

        commands.spawn((
            Mesh3d(assets.particle_mesh.clone()),
            MeshMaterial3d(assets.particle_material.clone()),
            Transform::from_translation(center),
            HitParticle {
                velocity: Vec3::new(vx, vy, vz),
                timer: Timer::from_seconds(0.18, TimerMode::Once),
            },
            MultishotUltimateEntity,
        ));
    }
}

fn spawn_backboard_wall(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    // Matte concrete/plaster backboard wall matching the Open Horizon aesthetic
    let backboard_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(0xD6, 0xD9, 0xDE),
        perceptual_roughness: 0.92,
        metallic: 0.01,
        reflectance: 0.2,
        ..default()
    });

    // Subtle soft ambient occlusion shadow strip at the wall-floor intersection
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
        MultishotUltimateEntity,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(WALL_WIDTH + 0.4, 0.002, 0.4))),
        MeshMaterial3d(shadow_mat),
        Transform::from_xyz(0.0, 0.0018, WALL_Z),
        MultishotUltimateEntity,
    ));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut stats: ResMut<MultishotUltimateStats>,
    mut rng: ResMut<UltimateRng>,
    settings: Option<Res<Settings>>,
) {
    *stats = MultishotUltimateStats::default();

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

    let assets = MultishotAssets {
        sphere_mesh: meshes.add(Sphere::new(1.0).mesh().uv(32, 18)),
        particle_mesh: meshes.add(Sphere::new(0.035)),
        target_material,
        particle_material,
    };

    // Start with 1 target; subsequent targets ramp up sequentially via spawn_timer to 5.
    let initial_pos = pick_separated_pos(&[], &mut rng);
    spawn_single_target(&mut commands, &assets, initial_pos, &mut rng);

    commands.insert_resource(assets);
}

fn sync_target_material(
    settings: Option<Res<Settings>>,
    assets: Option<Res<MultishotAssets>>,
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

#[allow(clippy::too_many_arguments)]
fn update_shrinking_targets(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<MultishotAssets>,
    mut score: ResMut<RoundScore>,
    mut stats: ResMut<MultishotUltimateStats>,
    mut rng: ResMut<UltimateRng>,
    mut targets: Query<(Entity, &mut Transform, &mut Target, &mut MultishotTarget)>,
) {
    let mut remaining_positions = Vec::new();

    for (entity, mut transform, mut target, mut multishot) in &mut targets {
        multishot.lifetime.tick(time.delta());

        if multishot.lifetime.is_finished() {
            commands.entity(entity).despawn();
            score.points -= 3;
            stats.expired_targets += 1;
        } else {
            let frac = multishot.lifetime.fraction();
            // Aimlabs Multishot Ultimate cycle:
            // 0.0 -> 0.30: Smooth ease-out expansion from tiny dot to peak radius
            // 0.30 -> 1.0: Smooth contraction down to tiny dot before despawning
            let new_radius = if frac < GROWTH_PHASE_FRACTION {
                let t = frac / GROWTH_PHASE_FRACTION;
                let eased = 1.0 - (1.0 - t) * (1.0 - t);
                (SPAWN_RADIUS + (multishot.max_radius - SPAWN_RADIUS) * eased).max(SPAWN_RADIUS)
            } else {
                let t = (frac - GROWTH_PHASE_FRACTION) / (1.0 - GROWTH_PHASE_FRACTION);
                (multishot.max_radius * (1.0 - t)).max(MIN_CONTRACT_RADIUS)
            };
            multishot.current_radius = new_radius;

            transform.scale = Vec3::splat(new_radius);
            target.shape = TargetShape::Sphere { radius: new_radius };

            remaining_positions.push(transform.translation);
        }
    }

    // Ramp up sequentially one target at a time up to ACTIVE_TARGET_COUNT (5).
    if remaining_positions.len() < ACTIVE_TARGET_COUNT {
        stats.spawn_timer.tick(time.delta());
        if stats.spawn_timer.is_finished() {
            let pos = pick_separated_pos(&remaining_positions, &mut rng);
            spawn_single_target(&mut commands, &assets, pos, &mut rng);
        }
    }
}

fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut HitParticle)>,
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
    assets: Res<MultishotAssets>,
    mut hits: MessageReader<TargetHit>,
    mut score: ResMut<RoundScore>,
    mut stats: ResMut<MultishotUltimateStats>,
    mut rng: ResMut<UltimateRng>,
    targets: Query<(Entity, &Transform, &MultishotTarget, &ReactionTimer)>,
) {
    for hit in hits.read() {
        let Ok((entity, transform, multishot, reaction)) = targets.get(hit.entity) else {
            continue;
        };

        // Speed/size bonus: hitting targets near peak size grants up to 1.5x points
        let size_ratio = (multishot.current_radius / multishot.max_radius).clamp(0.1, 1.0);
        let bonus = 0.8 + 0.7 * size_ratio;
        let points = (10.0 * bonus).round() as i32;

        score.points += points;
        score.hits += 1;
        stats.targets_destroyed += 1;
        stats.total_ttk_ms += reaction.elapsed_ms;

        let hit_pos = transform.translation;
        commands.entity(entity).despawn();

        spawn_hit_particles(&mut commands, &assets, hit_pos, &mut rng);

        let mut existing_positions = Vec::new();
        for (t_ent, t_trans, _, _) in &targets {
            if t_ent != entity {
                existing_positions.push(t_trans.translation);
            }
        }

        let new_pos = pick_separated_pos(&existing_positions, &mut rng);
        spawn_single_target(&mut commands, &assets, new_pos, &mut rng);
    }
}

fn handle_misses(mut misses: MessageReader<Miss>, mut score: ResMut<RoundScore>) {
    for _ in misses.read() {
        score.points -= 2;
        score.misses += 1;
    }
}

fn cleanup(
    mut commands: Commands,
    query: Query<Entity, With<MultishotUltimateEntity>>,
    mut stats: ResMut<MultishotUltimateStats>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    *stats = MultishotUltimateStats::default();
}
