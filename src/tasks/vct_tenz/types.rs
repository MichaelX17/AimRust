//! Types, constants, components, and RNG for the VCT NA - TenZ corridor rush task.

use bevy::prelude::*;

/// Number of targets allowed to pass the player line before the task ends immediately.
pub const MAX_BREACHES: usize = 3;

pub const TARGET_RADIUS: f32 = 0.24;
pub const MAX_ACTIVE_TARGETS: usize = 10;

pub const SPAWN_X_HALF: f32 = 3.5;
pub const SPAWN_Y_MIN: f32 = 0.65;
pub const SPAWN_Y_MAX: f32 = 3.95;
pub const MIN_PORTAL_SEP: f32 = 1.15;

pub const BASE_RUSH_SPEED: f32 = 7.0;
pub const RAMP_RUSH_SPEED: f32 = 8.0;
pub const BASE_SPAWN_INTERVAL: f32 = 0.90;
pub const MIN_SPAWN_INTERVAL: f32 = 0.32;
pub const RAMP_DURATION_SECS: f32 = 60.0;
pub const GATE_FLASH_SECS: f32 = 0.28;

/// Marker for every task-owned entity (rushing orbs and hit pop particles).
#[derive(Component)]
pub struct TenZEntity;

/// Marks an active target rushing straight down the corridor toward the player line.
#[derive(Component)]
pub struct TenZTarget {
    pub speed: f32,
}

/// Short-lived hit burst particle tinted by `TargetMaterial`.
#[derive(Component)]
pub struct TenZPopParticle {
    pub velocity: Vec3,
    pub timer: Timer,
}

/// Live round state for VCT NA - TenZ.
#[derive(Resource, Default)]
pub struct TenZState {
    pub elapsed: f32,
    pub spawn_timer: f32,
    pub breaches: usize,
    pub gate_flash_timer: f32,
    pub last_spawn_xy: Option<Vec2>,
}

/// Deterministic fast xorshift64 RNG for portal spawn coordinates.
#[derive(Resource)]
pub struct TenZRng(u64);

impl Default for TenZRng {
    fn default() -> Self {
        Self(0x7E12_C0DE_91A4_5F3B)
    }
}

impl TenZRng {
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
