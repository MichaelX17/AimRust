use bevy::prelude::*;

use crate::core::hit::{AimingAt, Miss, TargetExpired, TargetHit};
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, SelectedTask, TaskRegistry};

/// Score accumulated during the current round.
#[derive(Resource, Default)]
pub struct RoundScore {
    pub points: i32,
    pub hits: u32,
    pub misses: u32,
    pub tracking_ms: f64,
}

/// Scoring mode of the currently selected task (falls back to Click).
pub fn scoring_mode(registry: &TaskRegistry, selected: &SelectedTask) -> ScoringMode {
    registry
        .0
        .iter()
        .find(|task| task.id == selected.0)
        .map(|task| task.scoring)
        .unwrap_or(ScoringMode::Click)
}

/// Click-mode accuracy in percent (0 when no shots).
pub fn click_accuracy(score: &RoundScore) -> f32 {
    let shots = score.hits + score.misses;
    if shots == 0 {
        0.0
    } else {
        score.hits as f32 / shots as f32 * 100.0
    }
}

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoundScore>()
            .add_systems(OnEnter(AppState::Countdown), reset_score)
            .add_systems(
                Update,
                (click_scoring, tracking_scoring).run_if(in_state(AppState::Playing)),
            );
    }
}

fn reset_score(mut score: ResMut<RoundScore>) {
    *score = RoundScore::default();
}

fn click_scoring(
    mut hits: MessageReader<TargetHit>,
    mut misses: MessageReader<Miss>,
    mut expired: MessageReader<TargetExpired>,
    registry: Res<TaskRegistry>,
    selected: Res<SelectedTask>,
    mut score: ResMut<RoundScore>,
) {
    if scoring_mode(&registry, &selected) != ScoringMode::Click {
        return;
    }

    for hit in hits.read() {
        if hit.hit_zone.is_none() {
            score.points += 10;
            score.hits += 1;
        }
    }
    for _ in misses.read() {
        score.points -= 3;
        score.misses += 1;
    }
    for _ in expired.read() {
        score.points -= 3;
        score.misses += 1;
    }
}

fn tracking_scoring(
    time: Res<Time>,
    aiming: Res<AimingAt>,
    registry: Res<TaskRegistry>,
    selected: Res<SelectedTask>,
    mut score: ResMut<RoundScore>,
) {
    if scoring_mode(&registry, &selected) != ScoringMode::Tracking {
        return;
    }
    if aiming.0.is_some() {
        score.tracking_ms += time.delta_secs() as f64 * 1000.0;
        score.points = score.tracking_ms.floor() as i32;
    }
}
