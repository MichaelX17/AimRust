use bevy::prelude::*;

pub mod environment;
pub mod gun;
pub mod targets;

/// Visual assets (room, targets, gun).
pub struct PropsPlugin;

impl Plugin for PropsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(gun::GunPlugin)
            .add_systems(
            Startup,
            (
                environment::create_room_materials,
                targets::create_target_material,
            ),
        )
        .add_systems(
            Update,
            (
                environment::update_room_materials,
                targets::update_target_material,
            ),
        );
    }
}
