use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WallDepth {
    Close,
    Mid,
    Far,
}

impl WallDepth {
    pub fn z(&self) -> f32 {
        match self {
            WallDepth::Close => -10.0,
            WallDepth::Mid => -18.0,
            WallDepth::Far => -26.0,
        }
    }
}

/// Distance from screen center to the inner edge of each cover wall.
/// Creates a 7.0-meter central lane (from -3.5 to +3.5) with wide tactical separation.
pub const WALL_EDGE_X: f32 = 3.5;

#[derive(Resource)]
pub struct PeekExerciseState {
    pub remaining_depths: Vec<WallDepth>,
    pub current_depth: WallDepth,
    pub first_side_is_left: bool,
    pub peeks_completed_in_set: u8,
    pub cooldown_timer: Timer,
    pub active_target: Option<Entity>,
    pub remaining_styles: Vec<PeekStyle>,
}

impl Default for PeekExerciseState {
    fn default() -> Self {
        Self {
            remaining_depths: vec![WallDepth::Mid, WallDepth::Far],
            current_depth: WallDepth::Close,
            first_side_is_left: true,
            peeks_completed_in_set: 0,
            cooldown_timer: Timer::from_seconds(1.0, TimerMode::Once),
            active_target: None,
            remaining_styles: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeekStyle {
    CloseFast,
    CloseSlow,
    WideFast,
    WideSlow,
    WideThenClose,
    JumpPeek,
}

/// Shared feedback and base materials for peek defense targets.
#[derive(Resource)]
pub struct PeekMaterials {
    pub target: Handle<StandardMaterial>,
    pub headshot: Handle<StandardMaterial>,
    pub despawn: Handle<StandardMaterial>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PeekPhase {
    Out,
    CounterStrafe,
    Hold(Timer),
    Retreat,
    Feedback {
        timer: Timer,
        is_headshot: bool,
    },
    Done,
}

#[derive(Component)]
#[allow(dead_code)]
pub struct PeekTarget {
    pub style: PeekStyle,
    pub side_is_left: bool,
    pub wall_edge_x: f32,
    pub depth_z: f32,
    pub lifetime_timer: Timer,
    pub phase: PeekPhase,
    pub start_x: f32,
    pub apex_x: f32,
    pub counter_x: f32,
    pub speed_out: f32,
    pub speed_retreat: f32,
    pub jump_time: f32,
}

#[derive(Component)]
pub struct LeftWall;

#[derive(Component)]
pub struct RightWall;

#[derive(Component)]
pub struct PeekExerciseEntity;

#[derive(Resource)]
pub struct PeekRng(pub u64);

impl Default for PeekRng {
    fn default() -> Self {
        Self(0xA842_8F13_B69C_54E7)
    }
}

impl PeekRng {
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    pub fn gen_bool(&mut self) -> bool {
        (self.next_u64() & 1) == 1
    }

    pub fn gen_range(&mut self, min: usize, max: usize) -> usize {
        if min >= max {
            return min;
        }
        let range = (max - min) as u64;
        min + (self.next_u64() % range) as usize
    }

    pub fn shuffle_depths(&mut self, list: &mut [WallDepth]) {
        for i in (1..list.len()).rev() {
            let j = self.gen_range(0, i + 1);
            list.swap(i, j);
        }
    }

    pub fn shuffle_styles(&mut self, list: &mut [PeekStyle]) {
        for i in (1..list.len()).rev() {
            let j = self.gen_range(0, i + 1);
            list.swap(i, j);
        }
    }
}
