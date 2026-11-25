use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::paths;
use crate::core::round::RoundTimer;
use crate::core::score::RoundScore;
use crate::core::state::AppState;
use crate::core::task_api::SelectedTask;

const SCORES_FILE: &str = "scores.toml";
const MAX_RECENT: usize = 28;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoundRecord {
    pub unix_secs: u64,
    pub duration: u32,
    pub score: i32,
    pub hits: u32,
    pub misses: u32,
    pub tracking_ms: u64,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct BestRecord {
    pub score: i32,
    pub unix_secs: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct TaskScores {
    pub recent: Vec<RoundRecord>,
    pub best: BTreeMap<String, BestRecord>,
}

/// Stored scores: task id -> records.
#[derive(Resource, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Scores {
    pub tasks: BTreeMap<String, TaskScores>,
}

/// Outcome of the round that just finished.
#[derive(Resource, Default)]
pub struct LastOutcome {
    pub new_best: bool,
    pub best: i32,
}

pub struct StoragePlugin;

impl Plugin for StoragePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Scores>()
            .init_resource::<LastOutcome>()
            .add_systems(Startup, load_scores)
            .add_systems(OnEnter(AppState::Results), record_results);
    }
}

fn scores_path() -> Option<PathBuf> {
    paths::config_file_path(SCORES_FILE)
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("toml.bak")
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn load_scores(mut commands: Commands) {
    commands.insert_resource(read_scores());
}

fn read_scores() -> Scores {
    let Some(path) = scores_path() else {
        return Scores::default();
    };

    match std::fs::read_to_string(&path) {
        Ok(text) => match toml::from_str::<Scores>(&text) {
            Ok(scores) => scores,
            Err(_) => {
                let _ = std::fs::rename(&path, backup_path(&path));
                Scores::default()
            }
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Scores::default(),
        Err(_) => Scores::default(),
    }
}

/// Writes all scores; no-op on I/O failure.
pub fn save_scores(scores: &Scores) {
    let Some(path) = scores_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = toml::to_string_pretty(scores) {
        let _ = std::fs::write(&path, text);
    }
}

/// Appends the finished round, trims history, updates the best score for the
/// task+duration and writes the file once.
fn record_results(
    selected: Res<SelectedTask>,
    score: Res<RoundScore>,
    timer: Res<RoundTimer>,
    mut scores: ResMut<Scores>,
    mut last: ResMut<LastOutcome>,
) {
    let duration = timer.duration.round().max(0.0) as u32;
    let total_score = score.points.max(0);
    let record = RoundRecord {
        unix_secs: unix_now(),
        duration,
        score: total_score,
        hits: score.hits,
        misses: score.misses,
        tracking_ms: score.tracking_ms.floor() as u64,
    };

    let timestamp = record.unix_secs;
    let task = scores.tasks.entry(selected.0.to_string()).or_default();
    task.recent.push(record);
    if task.recent.len() > MAX_RECENT {
        task.recent.drain(0..task.recent.len() - MAX_RECENT);
    }

    let key = duration.to_string();
    let previous_best = task.best.get(&key).map(|best| best.score);
    match previous_best {
        Some(best) if total_score <= best => {
            last.new_best = false;
            last.best = best;
        }
        _ => {
            task.best.insert(
                key,
                BestRecord {
                    score: total_score,
                    unix_secs: timestamp,
                },
            );
            last.new_best = true;
            last.best = total_score;
        }
    }

    save_scores(&scores);
}
