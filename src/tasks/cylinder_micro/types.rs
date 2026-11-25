use bevy::prelude::*;

pub const TARGET_RADIUS: f32 = 0.20;
pub const CYLINDER_TARGET_RADIUS: f32 = 33.5;
pub const MIN_TARGET_SEPARATION: f32 = 6.0;
pub const ACTIVE_TARGET_COUNT: usize = 24;

#[derive(Component)]
pub struct CylinderMicroEntity;

#[derive(Component)]
pub struct MicroTarget;

#[derive(Resource)]
pub struct MicroRng(u64);

impl Default for MicroRng {
    fn default() -> Self {
        Self(0xCAFE_BABE_9876_5432)
    }
}

impl MicroRng {
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
