//! Gridshot: 3x3 grid of cells, always exactly 3 sphere targets.

use bevy::prelude::*;

use crate::core::hit::TargetHit;
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, TaskInfo, task_playing, task_selected};
use crate::props::targets::{TargetMaterial, spawn_sphere};

pub const INFO: TaskInfo = TaskInfo {
    id: "gridshot",
    name: "Gridshot",
    description: "3 targets on a 3x3 grid; hit to respawn",
    scoring: ScoringMode::Click,
};

const CELLS: usize = 9;
const SPACING: f32 = 1.2;
const CENTER_Y: f32 = 1.7;
const GRID_Z: f32 = -12.0;
const RADIUS: f32 = 0.3;
const TARGET_COUNT: usize = 3;

#[derive(Component)]
struct GridshotEntity;

#[derive(Component)]
struct GridCell(usize);

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
            .add_systems(Update, respawn_on_hit.run_if(task_playing(INFO.id)))
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
    commands.entity(entity).insert((GridshotEntity, GridCell(cell)));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut rng: ResMut<GridshotRng>,
) {
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
    targets: Query<(Entity, &GridCell), With<GridshotEntity>>,
) {
    for hit in hits.read() {
        let Some((_, vacated)) = targets.iter().find(|(entity, _)| *entity == hit.entity) else {
            continue;
        };
        let vacated = vacated.0;

        // Cells still occupied by the other targets after this one is gone.
        let mut occupied = [false; CELLS];
        for (entity, cell) in &targets {
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

fn cleanup(mut commands: Commands, targets: Query<Entity, With<GridshotEntity>>) {
    for entity in &targets {
        commands.entity(entity).despawn();
    }
}
