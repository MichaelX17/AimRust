use std::f32::consts::TAU;

use bevy::prelude::*;

pub const RADIUS: f32 = 0.3;
pub const PLANE_Z: f32 = -15.0;
pub const SPEED: f32 = 4.0;
pub const LIFETIME: f32 = 4.0;
pub const MIN_X: f32 = -5.0;
pub const MAX_X: f32 = 5.0;
pub const MIN_Y: f32 = 0.8;
pub const MAX_Y: f32 = 4.0;
pub const MIN_CHANGE: f32 = 0.5;
pub const MAX_CHANGE: f32 = 1.5;
pub const STEER_RATE: f32 = 6.0;
pub const MIN_RESPAWN_DISTANCE: f32 = 1.0;
pub const RESPAWN_ATTEMPTS: usize = 32;

// Backboard wall geometry on the Default arena
pub const WALL_WIDTH: f32 = 16.0;
pub const WALL_HEIGHT: f32 = 7.0;
pub const WALL_DEPTH: f32 = 0.3;
pub const WALL_Z: f32 = -15.6;

#[derive(Component)]
pub struct TrackingShootEntity;

#[derive(Component)]
pub struct TrackingShootTarget;

#[derive(Component)]
pub struct TrackingShootPopParticle {
    pub velocity: Vec3,
    pub timer: Timer,
}

#[derive(Component)]
pub struct TrackingShoot {
    pub velocity: Vec2,
    pub target_velocity: Vec2,
    pub change_in: f32,
}

#[derive(Resource)]
pub struct TrackingShootRng(u64);

impl Default for TrackingShootRng {
    fn default() -> Self {
        Self(0xA076_1D64_78BD_642F)
    }
}

impl TrackingShootRng {
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

    pub fn next_direction(&mut self) -> Vec2 {
        let angle = self.gen_f32() * TAU;
        Vec2::new(angle.cos(), angle.sin())
    }

    pub fn gen_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.gen_f32() * (max - min)
    }
}
