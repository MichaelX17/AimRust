//! Default Open-Horizon aim-training environment (without center backboard wall):
//! - Expansive matte dark-charcoal ground plane (#222428) stretching infinitely to the horizon
//! - Crisp, thin light-grey geometric grid lines forming uniform square tiles
//! - Distinct, thicker, brighter white centerline along Z-axis (X = 0)
//! - Clean, cloudless gradient sky dome (warm pale cream-yellow #EAE8C9 at horizon to medium sky-blue #6B93C4 overhead)
//! - Bright, even outdoor directional & ambient lighting

use bevy::prelude::*;

use crate::core::arena::ArenaEntity;
use crate::world::open_horizon_arena::build_sky_gradient_mesh;

/// Spawns the default Open-Horizon 3D aim-training environment without the center wall:
/// 1. Floor & Spatial Grid Geometry
/// 2. Atmospheric Gradient Skybox & Lighting
pub fn spawn_default_arena(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    // -------------------------------------------------------------------------
    // 1. Materials
    // -------------------------------------------------------------------------

    // Ground Plane: Expansive completely flat matte dark-charcoal (#222428)
    let floor_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(0x22, 0x24, 0x28),
        perceptual_roughness: 0.98,
        metallic: 0.0,
        reflectance: 0.05,
        ..default()
    });

    // Grid Overlay: Crisp, thin light-grey / off-white geometric grid lines
    let grid_line_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.52, 0.55, 0.60),
        perceptual_roughness: 0.85,
        metallic: 0.0,
        ..default()
    });

    // Center Alignment Line: Distinct, thicker, brighter white centerline along Z-axis (X = 0)
    let center_line_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.96, 0.97, 1.0),
        perceptual_roughness: 0.5,
        metallic: 0.0,
        emissive: LinearRgba::rgb(0.3, 0.32, 0.35),
        ..default()
    });

    // Unlit sky material honoring per-vertex gradient colors
    let sky_mat = materials.add(StandardMaterial {
        unlit: true,
        perceptual_roughness: 1.0,
        cull_mode: None,
        ..default()
    });

    // -------------------------------------------------------------------------
    // 2. Floor Plane & Spatial Grid
    // -------------------------------------------------------------------------
    let floor_size = 400.0;
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(floor_size * 0.5)))),
        MeshMaterial3d(floor_mat),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ArenaEntity,
    ));

    // Grid lines: 1m spacing over a 40m x 40m training area (X: -20..=20, Z: -30..=10)
    let grid_x_min = -20.0f32;
    let grid_x_max = 20.0f32;
    let grid_z_min = -30.0f32;
    let grid_z_max = 10.0f32;
    let grid_w = grid_x_max - grid_x_min;
    let grid_d = grid_z_max - grid_z_min;
    let grid_z_center = (grid_z_min + grid_z_max) * 0.5;

    // Transverse lines (parallel to X-axis, perpendicular to player view)
    let trans_mesh = meshes.add(Cuboid::new(grid_w, 0.002, 0.035));
    for z in (-30..=10).step_by(1) {
        commands.spawn((
            Mesh3d(trans_mesh.clone()),
            MeshMaterial3d(grid_line_mat.clone()),
            Transform::from_xyz(0.0, 0.001, z as f32),
            ArenaEntity,
        ));
    }

    // Longitudinal lines (parallel to Z-axis)
    let long_mesh = meshes.add(Cuboid::new(0.035, 0.002, grid_d));
    for x in (-20..=20).step_by(1) {
        if x == 0 {
            continue; // Center line has custom mesh/material
        }
        commands.spawn((
            Mesh3d(long_mesh.clone()),
            MeshMaterial3d(grid_line_mat.clone()),
            Transform::from_xyz(x as f32, 0.001, grid_z_center),
            ArenaEntity,
        ));
    }

    // Center Alignment Line: thicker (0.08m wide), brighter white, spanning the full grid along Z-axis
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.08, 0.003, grid_d))),
        MeshMaterial3d(center_line_mat),
        Transform::from_xyz(0.0, 0.0015, grid_z_center),
        ArenaEntity,
    ));

    // -------------------------------------------------------------------------
    // 3. Skybox Dome & Lighting
    // -------------------------------------------------------------------------
    // Horizon: warm, pale cream-yellow glow (#EAE8C9 -> [0.918, 0.910, 0.788, 1.0])
    // Mid: soft pale cyan (#C2DFF0 -> [0.761, 0.875, 0.941, 1.0])
    // Overhead: clear, calm medium sky-blue (#6B93C4 -> [0.420, 0.576, 0.769, 1.0])
    let horizon_color = [0.918, 0.910, 0.788, 1.0];
    let mid_color = [0.761, 0.875, 0.941, 1.0];
    let zenith_color = [0.420, 0.576, 0.769, 1.0];

    let sky_mesh = build_sky_gradient_mesh(
        350.0,
        64,
        32,
        horizon_color,
        mid_color,
        zenith_color,
    );

    commands.spawn((
        Mesh3d(meshes.add(sky_mesh)),
        MeshMaterial3d(sky_mat),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ArenaEntity,
    ));

    // Bright, even outdoor directional lighting
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: false,
            illuminance: 30_000.0, // Clean outdoor daylight
            color: Color::srgb(0.99, 0.98, 0.96),
            ..default()
        },
        Transform::from_xyz(2.0, 16.0, 8.0).looking_at(Vec3::new(0.0, 2.5, -10.0), Vec3::Y),
        ArenaEntity,
    ));
}
