//! Gridshot: 3x3 grid of cells, always exactly 3 sphere targets.
//! Follows the standard AimRust Default arena and target visual conventions.

use bevy::prelude::*;

use crate::core::camera::PlayerCamera;
use crate::core::hit::TargetHit;
use crate::core::hit_zone::ReactionTimer;
use crate::core::input::PlayerLook;
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, TaskInfo, task_playing, task_selected};
use crate::props::targets::{TargetMaterial, spawn_sphere};

pub const INFO: TaskInfo = TaskInfo {
    id: "gridshot",
    name: "Gridshot",
    description: "3 targets on a 3x3 grid; hit to respawn",
    scoring: ScoringMode::Click,
    is_sniper_mode: false,
};

const CELLS: usize = 9;
const SPACING: f32 = 1.3;
const CENTER_Y: f32 = 1.7;
const GRID_Z: f32 = -12.0;
const RADIUS: f32 = 0.32;
const TARGET_COUNT: usize = 3;

// Backboard wall geometry on the Default arena
const WALL_WIDTH: f32 = 16.0;
const WALL_HEIGHT: f32 = 7.0;
const WALL_DEPTH: f32 = 0.3;
const WALL_Z: f32 = -12.6;

#[derive(Component)]
struct GridshotEntity;

#[derive(Component)]
struct GridCell(usize);

#[derive(Component)]
struct GridshotPopParticle {
    velocity: Vec3,
    timer: Timer,
}

#[derive(Resource)]
struct GridshotRng(u64);

impl Default for GridshotRng {
    fn default() -> Self {
        Self(0x2545_F491_4F6C_DD1D)
    }
}

impl GridshotRng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn gen_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }

    fn gen_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.gen_f32() * (max - min)
    }
}

pub fn plugin() -> impl Plugin {
    GridshotPlugin
}

struct GridshotPlugin;

impl Plugin for GridshotPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GridshotRng>()
            .add_systems(
                OnEnter(AppState::Countdown),
                setup.run_if(task_selected(INFO.id)),
            )
            .add_systems(
                Update,
                (respawn_on_hit, update_pop_particles).run_if(task_playing(INFO.id)),
            )
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

fn cell_position(cell: usize) -> Vec3 {
    let col = (cell % 3) as f32;
    let row = (cell / 3) as f32;
    Vec3::new(
        (col - 1.0) * SPACING,
        CENTER_Y + (1.0 - row) * SPACING,
        GRID_Z,
    )
}

fn spawn_target(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    cell: usize,
) {
    let entity = spawn_sphere(
        commands,
        meshes,
        material,
        cell_position(cell),
        RADIUS,
        None,
    );
    commands
        .entity(entity)
        .insert((GridshotEntity, GridCell(cell), ReactionTimer::new()));
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
        GridshotEntity,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(WALL_WIDTH + 0.4, 0.002, 0.4))),
        MeshMaterial3d(shadow_mat),
        Transform::from_xyz(0.0, 0.0018, WALL_Z),
        GridshotEntity,
    ));
}

fn spawn_pop_particles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    pos: Vec3,
    rng: &mut GridshotRng,
) {
    let particle_mesh = meshes.add(Sphere::new(0.035));
    for _ in 0..6 {
        let vx = rng.gen_range(-2.0, 2.0);
        let vy = rng.gen_range(-1.2, 2.2);
        let vz = rng.gen_range(0.2, 2.0);

        commands.spawn((
            Mesh3d(particle_mesh.clone()),
            MeshMaterial3d(material.0.clone()),
            Transform::from_translation(pos),
            GridshotPopParticle {
                velocity: Vec3::new(vx, vy, vz),
                timer: Timer::from_seconds(0.14, TimerMode::Once),
            },
            GridshotEntity,
        ));
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material: Res<TargetMaterial>,
    mut rng: ResMut<GridshotRng>,
    mut look: ResMut<PlayerLook>,
    mut cameras: Query<&mut Transform, With<PlayerCamera>>,
) {
    *look = PlayerLook::default();
    if let Ok(mut cam_t) = cameras.single_mut() {
        cam_t.translation = Vec3::new(0.0, 1.7, 0.0);
        cam_t.rotation = Quat::IDENTITY;
    }

    spawn_backboard_wall(&mut commands, &mut meshes, &mut materials);

    let mut occupied = [false; CELLS];
    for _ in 0..TARGET_COUNT {
        let free: Vec<usize> = (0..CELLS).filter(|cell| !occupied[*cell]).collect();
        if free.is_empty() {
            break;
        }
        let cell = free[(rng.next_u64() % free.len() as u64) as usize];
        occupied[cell] = true;
        spawn_target(&mut commands, &mut meshes, &material, cell);
    }
}

fn respawn_on_hit(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut rng: ResMut<GridshotRng>,
    mut hits: MessageReader<TargetHit>,
    targets: Query<(Entity, &Transform, &GridCell), With<GridshotEntity>>,
) {
    for hit in hits.read() {
        let Some((_, hit_transform, hit_cell)) =
            targets.iter().find(|(entity, _, _)| *entity == hit.entity)
        else {
            continue;
        };
        let hit_pos = hit_transform.translation;
        let vacated = hit_cell.0;

        spawn_pop_particles(&mut commands, &mut meshes, &material, hit_pos, &mut rng);

        // Cells still occupied by the other targets after this one is gone.
        let mut occupied = [false; CELLS];
        for (entity, _, cell) in &targets {
            if entity != hit.entity {
                occupied[cell.0] = true;
            }
        }
        // Never respawn on the cell that was just vacated: that spot must be
        // used by a different spawn first.
        let free: Vec<usize> = (0..CELLS)
            .filter(|cell| !occupied[*cell] && *cell != vacated)
            .collect();
        if free.is_empty() {
            continue;
        }
        let cell = free[(rng.next_u64() % free.len() as u64) as usize];

        commands.entity(hit.entity).despawn();
        spawn_target(&mut commands, &mut meshes, &material, cell);
    }
}

fn update_pop_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut GridshotPopParticle)>,
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

fn cleanup(mut commands: Commands, entities: Query<Entity, With<GridshotEntity>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
