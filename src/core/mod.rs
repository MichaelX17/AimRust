use bevy::prelude::*;

pub mod arena;
pub mod camera;
pub mod config;
pub mod crosshair;
pub mod hud;
pub mod round;
pub mod score;
pub mod storage;
pub mod hit;
pub mod input;
pub mod state;
pub mod task_api;
pub mod ui;
pub mod video;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<state::AppState>()
            .init_resource::<task_api::TaskRegistry>()
            .init_resource::<task_api::SelectedTask>()
            .add_plugins((
                ui::UiPlugin,
                arena::ArenaPlugin,
                round::RoundPlugin,
                input::InputPlugin,
                crosshair::CrosshairPlugin,
                hit::HitPlugin,
                score::ScorePlugin,
                hud::HudPlugin,
                storage::StoragePlugin,
            ))
            .add_systems(Startup, (camera::spawn_player_camera, config::load_settings))
            .add_systems(
                Update,
                (config::autosave_settings, video::apply_video_settings),
            );
    }
}
