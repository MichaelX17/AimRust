use bevy::prelude::*;

// Other variants are entered by later tasks.
#[allow(dead_code)]
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    Menu,
    Settings,
    TaskSelect,
    TimeSelect,
    Countdown,
    Playing,
    Paused,
    Results,
}
