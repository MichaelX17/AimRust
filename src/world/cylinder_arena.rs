//! Giant vertical cylinder arena implementation for high-verticality and 360-degree aim training.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use crate::core::arena::ArenaEntity;

/// Procedural inward-facing cylinder wall mesh with correct interior winding,
/// inward normals, and clean tiled UVs.
pub fn build_inward_cylinder_mesh(radius: f32, height: f32, segments: u32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let seg_f = segments as f32;
    let circumference = radius * std::f32::consts::TAU;

    for y_idx in 0..=1 {
        let y = y_idx as f32 * height;
        let v = y_idx as f32 * (height / 10.0);
        for s in 0..=segments {
            let u = s as f32 / seg_f;
            let theta = u * std::f32::consts::TAU;
            let cos_t = theta.cos();
            let sin_t = theta.sin();

            positions.push([radius * cos_t, y, radius * sin_t]);
            normals.push([-cos_t, 0.0, -sin_t]);
            uvs.push([u * (circumference / 10.0), v]);
        }
    }

    let ring_stride = segments + 1;
    for s in 0..segments {
        let b0 = s;
        let b1 = s + 1;
        let t0 = b0 + ring_stride;
        let t1 = b1 + ring_stride;

        // Inward-facing winding: (b0, b1, t0) and (b1, t1, t0)
        indices.push(b0);
        indices.push(b1);
        indices.push(t0);

        indices.push(b1);
        indices.push(t1);
        indices.push(t0);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Spawns the giant vertical cylinder arena:
/// - Inward-facing cylindrical wall (radius 35m, height 120m)
/// - Matte dark-slate material for high target contrast
/// - Circular floor and ceiling caps
/// - Elevation reference rings every 10 meters of height
/// - Evenly distributed lighting from floor to ceiling
pub fn spawn_giant_cylinder_arena(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let radius = 35.0f32;
    let height = 120.0f32;
    let segments = 64u32;

    // Neutral dark-slate material
    let wall_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.13, 0.15),
        perceptual_roughness: 0.9,
        metallic: 0.05,
        ..default()
    });

    let floor_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.09, 0.10),
        perceptual_roughness: 0.95,
        metallic: 0.0,
        ..default()
    });

    let ceiling_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.06, 0.07, 0.08),
        perceptual_roughness: 0.95,
        metallic: 0.0,
        ..default()
    });

    let ring_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.24, 0.27, 0.32),
        perceptual_roughness: 0.6,
        metallic: 0.1,
        ..default()
    });

    // 1. Inward cylinder wall
    let wall_mesh = meshes.add(build_inward_cylinder_mesh(radius, height, segments));
    commands.spawn((
        Mesh3d(wall_mesh),
        MeshMaterial3d(wall_mat),
        Transform::IDENTITY,
        ArenaEntity,
    ));

    // 2. Floor cap (Circle lying in XZ plane at Y = 0 facing +Y)
    let circle_mesh = meshes.add(Circle::new(radius));
    commands.spawn((
        Mesh3d(circle_mesh.clone()),
        MeshMaterial3d(floor_mat),
        Transform::from_xyz(0.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ArenaEntity,
    ));

    // Ceiling cap (Circle lying in XZ plane at Y = height facing -Y)
    commands.spawn((
        Mesh3d(circle_mesh),
        MeshMaterial3d(ceiling_mat),
        Transform::from_xyz(0.0, height, 0.0)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        ArenaEntity,
    ));

    // 3. Elevation reference rings every 10m of height (Y = 10, 20, ..., 110)
    let ring_mesh = meshes.add(Torus {
        minor_radius: 0.04,
        major_radius: radius - 0.05,
    });

    let mut y = 10.0;
    while y < height {
        commands.spawn((
            Mesh3d(ring_mesh.clone()),
            MeshMaterial3d(ring_mat.clone()),
            Transform::from_xyz(0.0, y, 0.0),
            ArenaEntity,
        ));
        y += 10.0;
    }

    // 4. Lighting: directional daylight + central fill point light at Y = 60m
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: false,
            illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
            ..default()
        },
        Transform::from_xyz(0.0, 80.0, 0.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Z),
        ArenaEntity,
    ));

    commands.spawn((
        PointLight {
            shadow_maps_enabled: false,
            intensity: 800_000.0,
            range: 200.0,
            color: Color::srgb(0.95, 0.97, 1.0),
            ..default()
        },
        Transform::from_xyz(0.0, 60.0, 0.0),
        ArenaEntity,
    ));
}
