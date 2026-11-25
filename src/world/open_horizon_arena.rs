//! Open Horizon aim-training environment:
//! - Expansive matte dark-charcoal ground plane (#222428) stretching infinitely to the horizon
//! - Crisp, thin light-grey geometric grid lines forming uniform square tiles
//! - Distinct, thicker, brighter white centerline along Z-axis (X = 0) from beneath camera to wall
//! - Freestanding rectangular backboard wall (2.2:1 ratio) in matte concrete/plaster (#D6D9DE)
//! - Clean, cloudless gradient sky dome (warm pale cream-yellow #EAE8C9 at horizon to medium sky-blue #6B93C4 overhead)
//! - Bright, even outdoor directional & ambient lighting with subtle soft contact shadow at wall base

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use crate::core::arena::ArenaEntity;

/// Procedural hemisphere / dome mesh with inward-facing triangles and smooth vertex color
/// gradient from warm cream-yellow at horizon (phi = 0) to cyan to sky blue overhead (phi = pi/2).
pub fn build_sky_gradient_mesh(
    radius: f32,
    segments_u: u32,
    segments_v: u32,
    horizon_color: [f32; 4],
    mid_color: [f32; 4],
    zenith_color: [f32; 4],
) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let u_step = std::f32::consts::TAU / segments_u as f32;
    // Cover slightly below horizon (down to -0.05 rad) up to zenith (+pi/2 rad)
    let min_phi = -0.05f32;
    let max_phi = std::f32::consts::FRAC_PI_2;
    let phi_range = max_phi - min_phi;

    for v_idx in 0..=segments_v {
        let t = v_idx as f32 / segments_v as f32;
        let phi = min_phi + t * phi_range;
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();

        // 3-stop color interpolation:
        // t_norm = 0 at horizon (phi = 0), 1 at zenith (phi = pi/2)
        let t_norm = (phi / std::f32::consts::FRAC_PI_2).clamp(0.0, 1.0);
        let vertex_color = if t_norm < 0.35 {
            let k = t_norm / 0.35;
            lerp_color(horizon_color, mid_color, k)
        } else {
            let k = (t_norm - 0.35) / 0.65;
            lerp_color(mid_color, zenith_color, k)
        };

        for u_idx in 0..=segments_u {
            let theta = u_idx as f32 * u_step;
            let cos_theta = theta.cos();
            let sin_theta = theta.sin();

            // Point on sphere
            let x = radius * cos_phi * sin_theta;
            let y = radius * sin_phi;
            let z = radius * cos_phi * cos_theta;

            positions.push([x, y, z]);
            // Inward-facing normal
            normals.push([-cos_phi * sin_theta, -sin_phi, -cos_phi * cos_theta]);
            colors.push(vertex_color);
        }
    }

    let ring_stride = segments_u + 1;
    for v_idx in 0..segments_v {
        for u_idx in 0..segments_u {
            let b0 = v_idx * ring_stride + u_idx;
            let b1 = b0 + 1;
            let t0 = b0 + ring_stride;
            let t1 = t0 + 1;

            // Inward-facing winding: (b0, b1, t0) and (b1, t1, t0)
            indices.push(b0);
            indices.push(b1);
            indices.push(t0);

            indices.push(b1);
            indices.push(t1);
            indices.push(t0);
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn lerp_color(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

/// Spawns the Open-Horizon 3D aim-training environment:
/// 1. Floor & Spatial Grid Geometry
/// 2. Target Backboard Wall
/// 3. Atmospheric Gradient Skybox & Lighting
pub fn spawn_open_horizon_arena(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    // -------------------------------------------------------------------------
    // 1. Materials
    // -------------------------------------------------------------------------

    // Ground Plane: Expansive completely flat matte dark-charcoal (#222428)
    // sRGB(0x22, 0x24, 0x28) = sRGB(34, 36, 40)
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

    // Target Backboard Wall: Matte light-grey/off-white concrete or plaster finish (#D6D9DE)
    // sRGB(0xD6, 0xD9, 0xDE) = sRGB(214, 217, 222)
    let backboard_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(0xD6, 0xD9, 0xDE),
        perceptual_roughness: 0.92,
        metallic: 0.01,
        reflectance: 0.2,
        ..default()
    });

    // Subtle soft ambient occlusion shadow strip directly beneath the base of the backboard
    let shadow_mat = materials.add(StandardMaterial {
        base_color: Color::srgba_u8(0x0a, 0x0c, 0x0f, 0x99),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 1.0,
        metallic: 0.0,
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
            Transform::from_xyz(x as f32, 0.001, (grid_z_min + grid_z_max) * 0.5),
            ArenaEntity,
        ));
    }

    // Center Alignment Line: thicker (0.08m wide), brighter white, from camera (Z = 0.5) to wall (Z = -10.0)
    let center_length = 10.5f32;
    let center_z_pos = (0.5 + -10.0) * 0.5;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.08, 0.003, center_length))),
        MeshMaterial3d(center_line_mat),
        Transform::from_xyz(0.0, 0.0015, center_z_pos),
        ArenaEntity,
    ));

    // -------------------------------------------------------------------------
    // 3. Target Backboard Wall
    // -------------------------------------------------------------------------
    // Dimensions: 2.2:1 ratio (width: 11.0m, height: 5.0m, depth: 0.3m)
    // Resting flush on the floor (base at Y = 0.0, center at Y = 2.5)
    // Positioned at medium range along the center guideline (Z = -10.0)
    let wall_width = 11.0f32;
    let wall_height = 5.0f32;
    let wall_depth = 0.3f32;
    let wall_z = -10.0f32;

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(wall_width, wall_height, wall_depth))),
        MeshMaterial3d(backboard_mat),
        Transform::from_xyz(0.0, wall_height * 0.5, wall_z),
        ArenaEntity,
    ));

    // Soft contact shadow strip right at the base intersection with the floor
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(wall_width + 0.4, 0.002, 0.4))),
        MeshMaterial3d(shadow_mat),
        Transform::from_xyz(0.0, 0.0018, wall_z),
        ArenaEntity,
    ));

    // -------------------------------------------------------------------------
    // 4. Skybox Dome & Lighting
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

    // Bright, even outdoor directional lighting keeping front face uniformly lit
    // Angle: coming from slightly above and behind-left the player towards the wall
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
