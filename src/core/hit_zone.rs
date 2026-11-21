use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitZone {
    Head,
    Body,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct TargetHitbox {
    pub zone: HitZone,
}

#[derive(Component, Clone, Debug)]
pub struct ReactionTimer {
    pub timer: Timer,
    pub elapsed_ms: f32,
}

impl ReactionTimer {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(10.0, TimerMode::Once),
            elapsed_ms: 0.0,
        }
    }
}

impl Default for ReactionTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Ticks reaction timers on active targets.
pub fn tick_reaction_timers(time: Res<Time>, mut timers: Query<&mut ReactionTimer>) {
    let delta = time.delta();
    let delta_ms = delta.as_secs_f32() * 1000.0;
    for mut reaction in &mut timers {
        reaction.timer.tick(delta);
        reaction.elapsed_ms += delta_ms;
    }
}
