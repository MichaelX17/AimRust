use bevy::prelude::*;

pub const RADIUS: f32 = 0.25;
pub const PLANE_Z: f32 = -15.0;
pub const MIN_X: f32 = -6.0;
pub const MAX_X: f32 = 6.0;
pub const MIN_Y: f32 = 0.6;
pub const MAX_Y: f32 = 4.2;
pub const LIFETIME: f32 = 1.2;
pub const MIN_DELAY: f32 = 0.3;
pub const MAX_DELAY: f32 = 1.0;
pub const MIN_RESPAWN_DISTANCE: f32 = 1.0;
pub const RESPAWN_ATTEMPTS: usize = 32;

// Backboard wall geometry on the Default arena
pub const WALL_WIDTH: f32 = 18.0;
pub const WALL_HEIGHT: f32 = 7.5;
pub const WALL_DEPTH: f32 = 0.3;
pub const WALL_Z: f32 = -15.6;

#[derive(Component)]
pub struct FlickEntity;

#[derive(Component)]
pub struct FlickTarget;

#[derive(Component)]
pub struct FlickPopParticle {
    pub velocity: Vec3,
    pub timer: Timer,
}

#[derive(Resource, Default)]
pub struct FlickState {
    pub waiting: bool,
    pub timer: f32,
    pub last_position: Option<Vec3>,
}

#[derive(Resource)]
pub struct FlickRng(u64);

impl Default for FlickRng {
    fn default() -> Self {
        Self(0xE703_7ED1_A0B4_28DB)
    }
}

impl FlickRng {
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

    pub fn next_delay(&mut self) -> f32 {
        self.gen_range(MIN_DELAY, MAX_DELAY)
    }
}
