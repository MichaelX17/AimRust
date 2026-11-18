use bevy::prelude::*;
use bevy::window::{WindowPlugin, WindowResolution};

mod core;
mod props;
mod tasks;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "AimRust".into(),
                resolution: WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            crate::core::CorePlugin,
            crate::props::PropsPlugin,
            crate::tasks::TasksPlugin,
        ))
        .run();
}
