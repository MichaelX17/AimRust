//! Task-facing API. Items are consumed by task modules and later core tasks.
#![allow(dead_code)]

use bevy::prelude::*;

use crate::core::state::AppState;

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
