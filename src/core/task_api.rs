//! Task-facing API. Items are consumed by task modules and later core tasks.
#![allow(dead_code)]

use bevy::prelude::*;

use crate::core::hit_zone::HitZone;
use crate::core::score::RoundScore;
use crate::core::state::AppState;

/// Called when a target is hit to calculate points with hit zone multiplier and reaction bonus.
pub fn on_target_hit(score: &mut RoundScore, hit_zone: HitZone, reaction_ms: f32) {
    let base = match hit_zone {
        HitZone::Head => 20, // 2x points
        HitZone::Body => 10, // 1x points
    };
    let bonus = if reaction_ms < 200.0 { 5 } else { 0 };
    score.points += base + bonus;
    score.hits += 1;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoringMode {
    Click,
    Tracking,
}

#[derive(Clone, Copy, Debug)]
pub struct TaskInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub scoring: ScoringMode,
    pub is_sniper_mode: bool,
}

/// Helper to check if the active task has sniper mode enabled.
pub fn is_current_task_sniper(registry: &TaskRegistry, selected: &SelectedTask) -> bool {
    registry
        .0
        .iter()
        .find(|task| task.id == selected.0)
        .map(|task| task.is_sniper_mode)
        .unwrap_or(false)
}

/// All registered tasks, filled by `TasksPlugin`.
#[derive(Resource, Default)]
pub struct TaskRegistry(pub Vec<TaskInfo>);

/// Id of the task chosen in the task select screen.
#[derive(Resource, Default)]
pub struct SelectedTask(pub &'static str);

/// Run condition: `AppState::Playing` and the given task is selected.
pub fn task_playing(id: &'static str) -> impl FnMut(Res<State<AppState>>, Res<SelectedTask>) -> bool + Clone {
    move |state: Res<State<AppState>>, selected: Res<SelectedTask>| {
        *state.get() == AppState::Playing && selected.0 == id
    }
}

/// Run condition: the given task is selected, in any state.
pub fn task_selected(id: &'static str) -> impl FnMut(Res<SelectedTask>) -> bool + Clone {
    move |selected: Res<SelectedTask>| selected.0 == id
}
