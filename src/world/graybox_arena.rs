//! Graybox Arena inspired by the official Aimlabs training room layout
//! (specifically for Multishot Ultimate & wide-angle flicking scenarios).

use bevy::prelude::*;

use crate::core::arena::ArenaEntity;

/// Spawns the Aimlabs-inspired Graybox Arena:
/// - Enclosed industrial rectangular training room with wide first-person viewing angles
/// - Clean dark matte floor with geometric perimeter & layout lines
/// - Monolithic, stepped stone/concrete walls with recessed architectural panels
/// - Multi-tiered low perimeter platforms and ledges along back and side walls
/// - High industrial ceiling with structural support beams and recessed skylights
/// - Bright, uniform ambient and overhead skylight illumination with soft, non-distracting shadows
pub fn spawn_graybox_arena(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    // Monochromatic matte materials
    let floor_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.13, 0.14, 0.16),
        perceptual_roughness: 0.95,
        metallic: 0.02,
        ..default()
    });

    let grid_line_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.22, 0.25),
        perceptual_roughness: 0.9,
        metallic: 0.05,
        ..default()
    });

    let main_wall_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.19, 0.22),
        perceptual_roughness: 0.9,
        metallic: 0.03,
        ..default()
    });

    let recessed_panel_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.14, 0.15, 0.17),
        perceptual_roughness: 0.95,
        metallic: 0.02,
        ..default()
    });

    let platform_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.17, 0.19),
        perceptual_roughness: 0.9,
        metallic: 0.04,
        ..default()
    });

    let ceiling_beam_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.11, 0.12, 0.13),
        perceptual_roughness: 0.85,
        metallic: 0.1,
        ..default()
    });

    let skylight_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.90, 0.96),
        emissive: LinearRgba::rgb(0.8, 0.85, 0.95),
        perceptual_roughness: 0.3,
        metallic: 0.0,
        ..default()
    });

    // 1. Arena Floor (36m wide x 44m deep, top face at Y = 0.0)
    // Centered at Z = -16.0 (spanning Z from +6.0 behind player to -38.0 front wall)
    let floor_w = 36.0;
    let floor_d = 44.0;
    let floor_z = -16.0;

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(floor_w, 0.2, floor_d))),
        MeshMaterial3d(floor_mat.clone()),
        Transform::from_xyz(0.0, -0.1, floor_z),
        ArenaEntity,
    ));

    // Floor Grid / Measurement lines (subtle insets)
    for z in (-34..=4).step_by(4) {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(floor_w - 4.0, 0.01, 0.08))),
            MeshMaterial3d(grid_line_mat.clone()),
            Transform::from_xyz(0.0, 0.005, z as f32),
            ArenaEntity,
        ));
    }
    for x in (-14..=14).step_by(4) {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.08, 0.01, floor_d - 6.0))),
            MeshMaterial3d(grid_line_mat.clone()),
            Transform::from_xyz(x as f32, 0.005, floor_z),
            ArenaEntity,
        ));
    }

    // 2. High Ceiling (Y = 18.0)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(floor_w, 0.2, floor_d))),
        MeshMaterial3d(floor_mat),
        Transform::from_xyz(0.0, 18.1, floor_z),
        ArenaEntity,
    ));

    // Industrial Ceiling Beams & Recessed Skylights
    for z in [-28.0, -18.0, -8.0, 2.0] {
        // Transverse structural steel beams
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(floor_w, 0.8, 0.6))),
            MeshMaterial3d(ceiling_beam_mat.clone()),
            Transform::from_xyz(0.0, 17.6, z),
            ArenaEntity,
        ));

        // Overhead soft skylight panels
        for x in [-8.0, 8.0] {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(6.0, 0.1, 3.0))),
                MeshMaterial3d(skylight_mat.clone()),
                Transform::from_xyz(x, 17.95, z + 5.0),
                ArenaEntity,
            ));
        }
    }

    // 3. Front Wall (at Z = -38.0, height 18.0)
    let wall_h = 18.0;
    let wall_half_h = wall_h * 0.5;

    // Main front wall
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(floor_w, wall_h, 0.6))),
        MeshMaterial3d(main_wall_mat.clone()),
        Transform::from_xyz(0.0, wall_half_h, -38.3),
        ArenaEntity,
    ));

    // Architectural Recessed Panels along front wall
    for x in [-12.0, -6.0, 0.0, 6.0, 12.0] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(4.8, wall_h - 2.0, 0.3))),
            MeshMaterial3d(recessed_panel_mat.clone()),
            Transform::from_xyz(x, wall_half_h, -38.0),
            ArenaEntity,
        ));
    }

    // 4. Back Wall (at Z = 6.0 behind player)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(floor_w, wall_h, 0.6))),
        MeshMaterial3d(main_wall_mat.clone()),
        Transform::from_xyz(0.0, wall_half_h, 6.3),
        ArenaEntity,
    ));

    // 5. Left & Right Perimeter Walls (X = ±18.0)
    for x in [-18.3, 18.3] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.6, wall_h, floor_d))),
            MeshMaterial3d(main_wall_mat.clone()),
            Transform::from_xyz(x, wall_half_h, floor_z),
            ArenaEntity,
        ));

        // Side wall recessed architectural vertical bays
        let inner_x = if x < 0.0 { -18.0 } else { 18.0 };
        for z in [-30.0, -20.0, -10.0, 0.0] {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.3, wall_h - 2.0, 6.0))),
                MeshMaterial3d(recessed_panel_mat.clone()),
                Transform::from_xyz(inner_x, wall_half_h, z),
                ArenaEntity,
            ));
        }
    }

    // 6. Multi-Tiered Low Platforms & Ledges (along perimeter leaving center play area clear)
    // Low stepped side platform benches (Height 0.45m & 0.9m)
    for sign in [-1.0, 1.0] {
        let x = sign * 15.0;
        // Tier 1 bench
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(4.0, 0.45, 34.0))),
            MeshMaterial3d(platform_mat.clone()),
            Transform::from_xyz(x, 0.225, floor_z - 2.0),
            ArenaEntity,
        ));

        // Tier 2 rear platform ledge
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(3.0, 0.90, 12.0))),
            MeshMaterial3d(platform_mat.clone()),
            Transform::from_xyz(sign * 15.5, 0.45, -2.0),
            ArenaEntity,
        ));
    }

    // Front wall elevation baseline platform step (low riser 0.5m high, 2.5m deep)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(floor_w - 6.0, 0.5, 2.5))),
        MeshMaterial3d(platform_mat),
        Transform::from_xyz(0.0, 0.25, -36.5),
        ArenaEntity,
    ));

    // 7. Esports Uniform Clean Lighting Setup
    // Overhead daylight
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: false,
            illuminance: light_consts::lux::AMBIENT_DAYLIGHT * 1.2,
            ..default()
        },
        Transform::from_xyz(0.0, 16.0, -10.0).looking_at(Vec3::new(0.0, 0.0, -22.0), Vec3::Y),
        ArenaEntity,
    ));

    // Dual soft fill lights to ensure zero dark corners
    for z in [-30.0, -5.0] {
        commands.spawn((
            PointLight {
                shadow_maps_enabled: false,
                intensity: 600_000.0,
                range: 60.0,
                color: Color::srgb(0.96, 0.98, 1.0),
                ..default()
            },
            Transform::from_xyz(0.0, 14.0, z),
            ArenaEntity,
        ));
    }
}
