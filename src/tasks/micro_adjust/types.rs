use bevy::prelude::*;

// Compact backboard wall matching Micro_Adjust.mp4 framing (~18 deg wide x 9.7 deg tall at Z = -20)
pub const WALL_WIDTH: f32 = 6.4;
pub const WALL_HEIGHT: f32 = 3.4;
pub const WALL_DEPTH: f32 = 0.3;
pub const WALL_Z: f32 = -20.0;
pub const WALL_FRONT_Z: f32 = WALL_Z + WALL_DEPTH * 0.5; // -19.85

// Small 3D precision spheres hovering slightly forward of the wall (~0.63 deg diameter)
pub const TARGET_RADIUS: f32 = 0.11;
pub const TARGET_Z: f32 = WALL_FRONT_Z + 0.40; // -19.45

// Inner spawn bounds on the backboard with clean margins from edges
pub const X_MIN: f32 = -2.75;
pub const X_MAX: f32 = 2.75;
pub const Y_MIN: f32 = 0.45;
pub const Y_MAX: f32 = 2.95;

// Micro-adjustment clustering distances on the wall plane
pub const MIN_SEPARATION: f32 = 0.32;
pub const MICRO_MAX_SEPARATION: f32 = 1.15;
pub const WIDE_MAX_SEPARATION: f32 = 2.80;

pub const ACTIVE_TARGET_COUNT: usize = 2;
pub const RESPAWN_DELAY_SECS: f32 = 0.20;

#[derive(Component)]
pub struct MicroAdjustEntity;

#[derive(Component)]
pub struct MicroAdjustTarget;

#[derive(Component)]
pub struct MicroAdjustPopParticle {
    pub velocity: Vec3,
    pub timer: Timer,
}

#[derive(Resource)]
pub struct MicroAdjustAssets {
    pub sphere_mesh: Handle<Mesh>,
    pub particle_mesh: Handle<Mesh>,
    pub target_material: Handle<StandardMaterial>,
    pub particle_material: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct MicroAdjustStats {
    pub targets_destroyed: u32,
    pub streak: u32,
    pub total_ttk_ms: f32,
    pub pending_respawns: Vec<Timer>,
}

#[derive(Resource)]
pub struct MicroAdjustRng(u64);

impl Default for MicroAdjustRng {
    fn default() -> Self {
        Self(0xA5A5_5A5A_1234_5678)
    }
}

impl MicroAdjustRng {
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
