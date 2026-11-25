use bevy::prelude::*;

pub const MIN_TARGET_SEPARATION: f32 = 1.75;
pub const ACTIVE_TARGET_COUNT: usize = 5;

pub const SPAWN_RADIUS: f32 = 0.06;
pub const MIN_CONTRACT_RADIUS: f32 = 0.05;
pub const GROWTH_PHASE_FRACTION: f32 = 0.30;

// Backboard wall behind targets on the Default arena
pub const WALL_WIDTH: f32 = 24.0;
pub const WALL_HEIGHT: f32 = 8.5;
pub const WALL_DEPTH: f32 = 0.3;
pub const WALL_Z: f32 = -17.5;

#[derive(Component)]
pub struct MultishotUltimateEntity;

#[derive(Component)]
pub struct MultishotTarget {
    pub lifetime: Timer,
    pub max_radius: f32,
    pub current_radius: f32,
}

#[derive(Component)]
pub struct HitParticle {
    pub velocity: Vec3,
    pub timer: Timer,
}

#[derive(Resource)]
pub struct MultishotAssets {
    pub sphere_mesh: Handle<Mesh>,
    pub particle_mesh: Handle<Mesh>,
    pub target_material: Handle<StandardMaterial>,
    pub particle_material: Handle<StandardMaterial>,
}

#[derive(Resource)]
pub struct MultishotUltimateStats {
    pub targets_destroyed: u32,
    pub expired_targets: u32,
    pub total_ttk_ms: f32,
    pub spawn_timer: Timer,
}

impl Default for MultishotUltimateStats {
    fn default() -> Self {
        Self {
            targets_destroyed: 0,
            expired_targets: 0,
            total_ttk_ms: 0.0,
            spawn_timer: Timer::from_seconds(0.35, TimerMode::Repeating),
        }
    }
}

#[derive(Resource)]
pub struct UltimateRng(u64);

impl Default for UltimateRng {
    fn default() -> Self {
        Self(0xBEEF_F00D_1234_5678)
    }
}

impl UltimateRng {
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    pub fn gen_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }

    pub fn gen_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.gen_f32() * (max - min)
    }
}
