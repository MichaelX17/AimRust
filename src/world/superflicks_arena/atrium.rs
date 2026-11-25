//! Level 4 Sunlit Octagonal Sky Atrium:
//! Crisp white 8-sided walls, vibrant matte teal floor, concentric silver/cyan/charcoal
//! central landing pad, 12 perimeter black & cyan cylindrical bollards, giant skylight oculus,
//! and bright turquoise skybox with white cloud clusters overhead.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::core::arena::ArenaEntity;
use crate::world::open_horizon_arena::build_sky_gradient_mesh;

use super::meshes::{
    build_annulus_mesh, build_curved_ledge_mesh, build_polygonal_annulus_mesh,
    build_polygonal_wall_mesh,
};
use super::{
    ATRIUM_HEIGHT, ATRIUM_TURRET_COUNT, ATRIUM_TURRET_RING_RADIUS, ArenaPalette, LEVEL_RADII,
    LEVEL_Y, SHAFT_RADIUS,
};

const OCTAGON_SIDES: u32 = 8;
const OCULUS_RADIUS: f32 = 8.2;

/// Spawns the complete Level 4 Sunlit Octagonal Sky Atrium.
pub(super) fn spawn_level_4_atrium(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    palette: &ArenaPalette,
) {
    let floor_y = LEVEL_Y[3];
    let ceiling_y = floor_y + ATRIUM_HEIGHT;
    let radius = LEVEL_RADII[3];

    // 1. Flat-shaded 8-sided bright white octagonal atrium walls
    commands.spawn((
        Mesh3d(meshes.add(build_polygonal_wall_mesh(radius, ATRIUM_HEIGHT, OCTAGON_SIDES))),
        MeshMaterial3d(palette.atrium_wall.clone()),
        Transform::from_xyz(0.0, floor_y, 0.0),
        ArenaEntity,
    ));

    // 2. Vibrant matte teal floor surrounding the central multi-ring landing pad
    commands.spawn((
        Mesh3d(meshes.add(build_polygonal_annulus_mesh(
            4.55,
            radius + 0.08,
            OCTAGON_SIDES,
            8,
            true,
        ))),
        MeshMaterial3d(palette.atrium_teal.clone()),
        Transform::from_xyz(0.0, floor_y, 0.0),
        ArenaEntity,
    ));

    // 3. Multi-ring central landing pad (cyan -> charcoal -> silver -> cyan -> charcoal)
    spawn_landing_pad_rings(commands, meshes, palette, floor_y);

    // 4. 12 Perimeter matte black cylindrical bollards with twin cyan glow collars
    spawn_perimeter_bollards(commands, meshes, palette, floor_y);

    // 5. Ceiling with massive circular skylight oculus
    spawn_skylight_ceiling(commands, meshes, palette, ceiling_y, radius);

    // 6. Bright turquoise daylight sky dome and stylized white cloud clusters above oculus
    spawn_atrium_sky_and_clouds(commands, meshes, materials, palette, floor_y);
}

fn spawn_landing_pad_rings(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    palette: &ArenaPalette,
    floor_y: f32,
) {
    let rings: [(f32, f32, f32, &Handle<StandardMaterial>); 5] = [
        (SHAFT_RADIUS, SHAFT_RADIUS + 0.17, 0.015, &palette.cyan_glow),
        (SHAFT_RADIUS + 0.17, 3.06, 0.012, &palette.charcoal),
        (3.06, 4.00, 0.008, &palette.atrium_silver),
        (4.00, 4.20, 0.014, &palette.cyan_glow),
        (4.20, 4.56, 0.010, &palette.charcoal),
    ];

    for (inner_r, outer_r, y_offset, mat) in rings {
        commands.spawn((
            Mesh3d(meshes.add(build_annulus_mesh(inner_r, outer_r, 64, true))),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(0.0, floor_y + y_offset, 0.0),
            ArenaEntity,
        ));
    }
}

fn spawn_perimeter_bollards(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    palette: &ArenaPalette,
    floor_y: f32,
) {
    let cap_mesh = meshes.add(Cylinder::new(0.32, 0.14));
    let glow_mesh = meshes.add(Cylinder::new(0.285, 0.05));
    let body_mesh = meshes.add(Cylinder::new(0.245, 1.02));
    let mark_mesh = meshes.add(Cuboid::new(0.07, 0.24, 0.025));

    for i in 0..ATRIUM_TURRET_COUNT {
        // Offset by half-step (15 deg) so bollards frame the forward -Z wall corner symmetrically
        let angle = (i as f32 + 0.5) / ATRIUM_TURRET_COUNT as f32 * TAU;
        let x = ATRIUM_TURRET_RING_RADIUS * angle.sin();
        let z = -ATRIUM_TURRET_RING_RADIUS * angle.cos();

        let parent = commands
            .spawn((
                Transform::from_xyz(x, floor_y, z)
                    .looking_at(Vec3::new(0.0, floor_y, 0.0), Vec3::Y),
                Visibility::default(),
                ArenaEntity,
            ))
            .id();

        // Base disc
        commands.spawn((
            Mesh3d(cap_mesh.clone()),
            MeshMaterial3d(palette.charcoal.clone()),
            Transform::from_xyz(0.0, 0.07, 0.0),
            ChildOf(parent),
        ));
        // Lower cyan LED ring
        commands.spawn((
            Mesh3d(glow_mesh.clone()),
            MeshMaterial3d(palette.cyan_glow.clone()),
            Transform::from_xyz(0.0, 0.165, 0.0),
            ChildOf(parent),
        ));
        // Main dark cylindrical body
        commands.spawn((
            Mesh3d(body_mesh.clone()),
            MeshMaterial3d(palette.charcoal.clone()),
            Transform::from_xyz(0.0, 0.70, 0.0),
            ChildOf(parent),
        ));
        // Upper cyan LED ring
        commands.spawn((
            Mesh3d(glow_mesh.clone()),
            MeshMaterial3d(palette.cyan_glow.clone()),
            Transform::from_xyz(0.0, 1.235, 0.0),
            ChildOf(parent),
        ));
        // Top cap disc
        commands.spawn((
            Mesh3d(cap_mesh.clone()),
            MeshMaterial3d(palette.charcoal.clone()),
            Transform::from_xyz(0.0, 1.33, 0.0),
            ChildOf(parent),
        ));
        // Crisp white vertical accent bar facing inward toward the player (-Z in local space)
        commands.spawn((
            Mesh3d(mark_mesh.clone()),
            MeshMaterial3d(palette.bollard_accent.clone()),
            Transform::from_xyz(0.0, 0.72, -0.245),
            ChildOf(parent),
        ));
    }
}

fn spawn_skylight_ceiling(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    palette: &ArenaPalette,
    ceiling_y: f32,
    radius: f32,
) {
    // Octagonal-outer to circular-inner flat ceiling facing downward
    commands.spawn((
        Mesh3d(meshes.add(build_polygonal_annulus_mesh(
            OCULUS_RADIUS,
            radius + 0.08,
            OCTAGON_SIDES,
            8,
            false,
        ))),
        MeshMaterial3d(palette.atrium_ceiling.clone()),
        Transform::from_xyz(0.0, ceiling_y, 0.0),
        ArenaEntity,
    ));

    // Dark charcoal oculus bezel frame
    commands.spawn((
        Mesh3d(meshes.add(build_curved_ledge_mesh(
            OCULUS_RADIUS - 0.15,
            OCULUS_RADIUS + 0.30,
            0.38,
            0.0,
            TAU,
            64,
        ))),
        MeshMaterial3d(palette.charcoal.clone()),
        Transform::from_xyz(0.0, ceiling_y + 0.16, 0.0),
        ArenaEntity,
    ));

    // Bright cyan-blue inner accent rim lining the skylight opening
    commands.spawn((
        Mesh3d(meshes.add(build_curved_ledge_mesh(
            OCULUS_RADIUS - 0.24,
            OCULUS_RADIUS - 0.12,
            0.12,
            0.0,
            TAU,
            64,
        ))),
        MeshMaterial3d(palette.cyan_glow.clone()),
        Transform::from_xyz(0.0, ceiling_y - 0.12, 0.0),
        ArenaEntity,
    ));
}

fn spawn_atrium_sky_and_clouds(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    palette: &ArenaPalette,
    floor_y: f32,
) {
    let sky_mat = materials.add(StandardMaterial {
        unlit: true,
        perceptual_roughness: 1.0,
        cull_mode: None,
        ..default()
    });

    // Bright sunny turquoise-aqua gradient sky visible through the Level 4 oculus
    let sky_mesh = build_sky_gradient_mesh(
        95.0,
        48,
        24,
        [0.68, 0.96, 0.96, 1.0],
        [0.48, 0.90, 0.91, 1.0],
        [0.36, 0.83, 0.88, 1.0],
    );
    commands.spawn((
        Mesh3d(meshes.add(sky_mesh)),
        MeshMaterial3d(sky_mat),
        Transform::from_xyz(0.0, floor_y + 6.0, 0.0),
        ArenaEntity,
    ));

    // Stylized bright white puffy cloud clusters floating across the skylight opening
    let puff_mesh = meshes.add(Sphere::new(1.0).mesh().uv(20, 12));
    let puffs: [(Vec3, Vec3); 10] = [
        (Vec3::new(-4.5, 36.0, -7.5), Vec3::new(5.8, 2.4, 4.6)),
        (Vec3::new(-0.8, 37.2, -9.0), Vec3::new(6.5, 2.8, 5.2)),
        (Vec3::new(3.8, 36.5, -6.8), Vec3::new(5.2, 2.3, 4.4)),
        (Vec3::new(6.8, 38.0, 3.0), Vec3::new(6.0, 2.5, 5.0)),
        (Vec3::new(3.2, 39.2, 6.4), Vec3::new(5.0, 2.2, 4.2)),
        (Vec3::new(-6.2, 38.5, 4.8), Vec3::new(5.6, 2.4, 4.8)),
        (Vec3::new(-2.4, 39.8, 7.5), Vec3::new(4.8, 2.0, 4.0)),
        (Vec3::new(0.2, 41.0, -1.8), Vec3::new(4.2, 1.8, 3.6)),
        (Vec3::new(-8.5, 35.0, -2.0), Vec3::new(5.5, 2.2, 4.5)),
        (Vec3::new(8.5, 35.5, -3.5), Vec3::new(5.4, 2.2, 4.5)),
    ];

    for (offset, scale) in puffs {
        commands.spawn((
            Mesh3d(puff_mesh.clone()),
            MeshMaterial3d(palette.cloud_white.clone()),
            Transform::from_translation(Vec3::new(offset.x, floor_y + offset.y, offset.z))
                .with_scale(scale),
            ArenaEntity,
        ));
    }
}
