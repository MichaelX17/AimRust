//! Data types, constants, components, and RNG for the 4-phase SuperFlicks task.

use bevy::prelude::*;

/// Duration (seconds) of the smoothstep elevator ascent between adjacent levels.
pub const ELEVATOR_DURATION_SECS: f32 = 1.65;

/// Fraction of elevator ascent at which the next floor's targets appear as you emerge.
pub const SPAWN_AT_ELEVATOR_PROGRESS: f32 = 0.85;

// -----------------------------------------------------------------------------
// Phase 1 (Level 1): 5-Line Multi-Tier Fast Strafing Capsule Bots
// -----------------------------------------------------------------------------
pub const PHASE1_TARGET_COUNT: usize = 5;
/// Safe horizontal arc half-angles (deg) for each of the 5 Level 1 lines (curb + 4 ledges).
pub const PHASE1_LEDGE_HALF_DEGS: [f32; 5] = [125.0, 120.0, 110.0, 96.0, 68.0];
pub const PHASE1_ANGULAR_SPEED_MIN: f32 = 0.28;
pub const PHASE1_ANGULAR_SPEED_MAX: f32 = 0.56;
pub const PHASE1_STRAFE_HOLD_MIN: f32 = 0.8;
pub const PHASE1_STRAFE_HOLD_MAX: f32 = 1.9;
pub const PHASE1_MIN_ANGLE_SEP: f32 = 0.45;

// -----------------------------------------------------------------------------
// Phase 2 (Level 2): Fast Wide-Range Sweeping Medium Orbs
// -----------------------------------------------------------------------------
pub const PHASE2_TARGET_COUNT: usize = 10;
pub const PHASE2_RADIUS: f32 = 0.30;
pub const PHASE2_DIST_MIN: f32 = 10.2;
pub const PHASE2_DIST_MAX: f32 = 12.2;
pub const PHASE2_Y_MIN: f32 = 1.3;
pub const PHASE2_Y_MAX: f32 = 11.2;
pub const PHASE2_YAW_HALF_DEG: f32 = 135.0;
pub const PHASE2_ANGULAR_SPEED_MIN: f32 = 0.22;
pub const PHASE2_ANGULAR_SPEED_MAX: f32 = 0.42;
pub const PHASE2_VERTICAL_SPEED_MIN: f32 = 2.2;
pub const PHASE2_VERTICAL_SPEED_MAX: f32 = 4.6;
pub const PHASE2_HOLD_MIN: f32 = 0.75;
pub const PHASE2_HOLD_MAX: f32 = 1.85;
pub const PHASE2_MIN_SEP: f32 = 2.0;

// -----------------------------------------------------------------------------
// Phase 3 (Level 3): Wide-Spread Small High-Precision Static Micro Orbs
// -----------------------------------------------------------------------------
pub const PHASE3_TARGET_COUNT: usize = 12;
pub const PHASE3_RADIUS: f32 = 0.09;
pub const PHASE3_DIST_MIN: f32 = 11.6;
pub const PHASE3_DIST_MAX: f32 = 13.4;
pub const PHASE3_Y_MIN: f32 = 1.3;
pub const PHASE3_Y_MAX: f32 = 10.8;
pub const PHASE3_YAW_HALF_DEG: f32 = 155.0;
pub const PHASE3_MIN_SEP: f32 = 5.2;

// -----------------------------------------------------------------------------
// Phase 4 (Level 4): Sunlit Atrium Floor-Turret Ballistic Projectile Orbs
// -----------------------------------------------------------------------------
pub const PHASE4_MAX_TARGETS: usize = 5;
pub const PHASE4_RADIUS_MIN: f32 = 0.24;
pub const PHASE4_RADIUS_MAX: f32 = 0.30;
/// Gravity scaled by 1.25^2 so parabolic arcs match atrium height while flying 25% faster.
pub const PHASE4_GRAVITY: f32 = 13.75;
pub const PHASE4_LAUNCH_VY_MIN: f32 = 13.0;
pub const PHASE4_LAUNCH_VY_MAX: f32 = 16.25;
/// Sweeping left->right or right->left angular arc across the atrium (degrees).
pub const PHASE4_ARC_DEG_MIN: f32 = 65.0;
pub const PHASE4_ARC_DEG_MAX: f32 = 105.0;
pub const PHASE4_LAND_RADIUS_MIN: f32 = 10.6;
pub const PHASE4_LAND_RADIUS_MAX: f32 = 13.2;
pub const PHASE4_LAUNCH_DELAY_MIN: f32 = 0.28;
pub const PHASE4_LAUNCH_DELAY_MAX: f32 = 0.52;

/// Marks every task-owned entity (targets and hit pop particles) for cleanup.
#[derive(Component)]
pub struct SuperFlicksEntity;

/// Marks a root target entity owned by a specific phase (`0..=3`).
#[derive(Component)]
pub struct SuperFlicksTarget {
    pub phase: usize,
    /// Vertical offset from entity origin to visual center (for hit pop particles).
    pub hit_offset_y: f32,
}

/// Phase 1 curved-catwalk horizontal strafing behavior for capsule bots.
#[derive(Component)]
pub struct LedgeStrafe {
    pub tier: usize,
    pub angle: f32,
    pub angular_vel: f32,
    pub half_arc: f32,
    pub change_timer: f32,
}

/// Phase 2 fast wide-range sweeping horizontal & vertical motion on the cylindrical shell.
#[derive(Component)]
pub struct OrbSweep {
    pub radius: f32,
    pub angle: f32,
    pub y: f32,
    pub angular_vel: f32,
    pub vertical_vel: f32,
    pub yaw_limit: f32,
    pub y_min: f32,
    pub y_max: f32,
    pub change_timer: f32,
}

/// Phase 4 parabolic projectile motion launched from Level 4 perimeter floor turrets.
#[derive(Component)]
pub struct BallisticOrb {
    pub velocity: Vec3,
    pub radius: f32,
}

/// Short-lived hit burst particle that disperses and shrinks on target destruction.
#[derive(Component)]
pub struct SuperFlicksPopParticle {
    pub velocity: Vec3,
    pub timer: Timer,
}

/// Tracks active phase progression, elevator ascent, Phase 1 line positions, and Phase 4 turrets.
#[derive(Resource)]
pub struct SuperFlicksState {
    pub active_phase: usize,
    pub elevator_from_y: f32,
    pub elevator_to_y: f32,
    pub elevator_elapsed: f32,
    pub elevator_moving: bool,
    pub phase_spawned: bool,
    pub current_y: f32,
    pub phase1_last_pos: [Option<Vec3>; PHASE1_TARGET_COUNT],
    pub phase4_launch_timer: f32,
    pub last_turret: Option<usize>,
}

impl Default for SuperFlicksState {
    fn default() -> Self {
        Self {
            active_phase: 0,
            elevator_from_y: 0.0,
            elevator_to_y: 0.0,
            elevator_elapsed: 0.0,
            elevator_moving: false,
            phase_spawned: false,
            current_y: 0.0,
            phase1_last_pos: [None; PHASE1_TARGET_COUNT],
            phase4_launch_timer: 0.0,
            last_turret: None,
        }
    }
}

/// Deterministic fast xorshift64 RNG for SuperFlicks target placement and movement.
#[derive(Resource)]
pub struct SuperFlicksRng(u64);

impl Default for SuperFlicksRng {
    fn default() -> Self {
        Self(0x50F1_C4E5_9A37_11BD)
    }
}

impl SuperFlicksRng {
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

    pub fn gen_usize(&mut self, bound: usize) -> usize {
        if bound == 0 {
            return 0;
        }
        (self.next_u64() % bound as u64) as usize
    }

    pub fn gen_sign(&mut self) -> f32 {
        if (self.next_u64() & 1) == 0 { 1.0 } else { -1.0 }
    }
}
