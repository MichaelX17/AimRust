//! Copy this folder to `src/tasks/<name>/`, set `INFO`, then add `<name>` to
//! `register_tasks!(...)` in `src/tasks/mod.rs`.

use bevy::prelude::*;

use crate::core::task_api::{ScoringMode, TaskInfo};

pub const INFO: TaskInfo = TaskInfo {
    id: "template",
    name: "Template",
    description: "Copy of the task module template.",
    scoring: ScoringMode::Click,
};

pub fn plugin() -> impl Plugin {
    TemplatePlugin
}

struct TemplatePlugin;

impl Plugin for TemplatePlugin {
    fn build(&self, _app: &mut App) {
        // Round scene setup (spawn targets/props) when this task's countdown starts:
        // _app.add_systems(OnEnter(AppState::Countdown), setup.run_if(task_selected(INFO.id)));
        //
        // Gameplay systems, only while this task is playing:
        // _app.add_systems(Update, (tick, respawn).run_if(task_playing(INFO.id)));
        //
        // Cleanup on leaving the round (both paths):
        // _app.add_systems(OnEnter(AppState::Results), cleanup);
        // _app.add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

#[allow(dead_code)]
fn setup() {
    // Spawn this task's targets using props builders; never touch config/menus/scoring.
}

#[allow(dead_code)]
fn tick() {
    // React to TargetHit / Miss / TargetExpired, AimingAt, or move targets.
}
