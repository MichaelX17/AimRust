use bevy::prelude::*;

use crate::core::arena::ArenaEntity;
use crate::core::config::Settings;

/// Shared materials for the room surfaces.
#[derive(Resource)]
pub struct RoomMaterials {
    pub floor: Handle<StandardMaterial>,
    pub ceiling: Handle<StandardMaterial>,
    pub walls: Handle<StandardMaterial>,
}

fn surface(rgb: [u8; 3]) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb_u8(rgb[0], rgb[1], rgb[2]),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        unlit: true,
        ..default()
    }
}

/// Creates the shared room materials. Falls back to default colors when the
/// settings resource is not available yet (it is corrected on the next update).
pub fn create_room_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Option<Res<Settings>>,
) {
    let colors = settings
        .as_ref()
        .map(|settings| settings.colors.clone())
        .unwrap_or_default();

    commands.insert_resource(RoomMaterials {
        floor: materials.add(surface(colors.floor)),
        ceiling: materials.add(surface(colors.ceiling)),
        walls: materials.add(surface(colors.walls)),
    });
}

/// Recolors the shared materials when the configured colors change.
pub fn update_room_materials(
    settings: Res<Settings>,
    room: Option<Res<RoomMaterials>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !settings.is_changed() {
        return;
    }
    let Some(room) = room else {
        return;
    };

    set_color(&mut materials, &room.floor, settings.colors.floor);
    set_color(&mut materials, &room.ceiling, settings.colors.ceiling);
    set_color(&mut materials, &room.walls, settings.colors.walls);
}

fn set_color(
    materials: &mut Assets<StandardMaterial>,
    handle: &Handle<StandardMaterial>,
    rgb: [u8; 3],
) {
    if let Some(mut material) = materials.get_mut(handle) {
        material.base_color = Color::srgb_u8(rgb[0], rgb[1], rgb[2]);
    }
}

/// Spawns the room shell: floor, ceiling and four walls.
///
/// Interior bounds: x -10..10, y 0..6, z -30..5.
pub fn spawn_room(commands: &mut Commands, meshes: &mut Assets<Mesh>, materials: &RoomMaterials) {
    let floor_mesh = meshes.add(Cuboid::new(20.0, 0.1, 35.0));
    let wall_x_mesh = meshes.add(Cuboid::new(20.0, 6.0, 0.1));
    let wall_z_mesh = meshes.add(Cuboid::new(0.1, 6.0, 35.0));

    // Floor (top face at y = 0) and ceiling (bottom face at y = 6).
    commands.spawn((
        Mesh3d(floor_mesh.clone()),
        MeshMaterial3d(materials.floor.clone()),
        Transform::from_xyz(0.0, -0.05, -12.5),
        ArenaEntity,
    ));
    commands.spawn((
        Mesh3d(floor_mesh),
        MeshMaterial3d(materials.ceiling.clone()),
        Transform::from_xyz(0.0, 6.05, -12.5),
        ArenaEntity,
    ));

    // Front/back walls (inner faces at z = 5 and z = -30).
    for z in [5.05, -30.05] {
        commands.spawn((
            Mesh3d(wall_x_mesh.clone()),
            MeshMaterial3d(materials.walls.clone()),
            Transform::from_xyz(0.0, 3.0, z),
            ArenaEntity,
        ));
    }

    // Side walls (inner faces at x = -10 and x = 10).
    for x in [-10.05, 10.05] {
        commands.spawn((
            Mesh3d(wall_z_mesh.clone()),
            MeshMaterial3d(materials.walls.clone()),
            Transform::from_xyz(x, 3.0, -12.5),
            ArenaEntity,
        ));
    }
}
