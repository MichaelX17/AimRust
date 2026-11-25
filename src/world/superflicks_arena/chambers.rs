//! Lower 3 cylindrical chambers, Level 1 curved catwalk ledges, vertical fluted shafts,
//! overhead gantry pods, and the moving central circular elevator platform.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::core::arena::ArenaEntity;
use crate::world::cylinder_arena::build_inward_cylinder_mesh;

use super::meshes::{build_annulus_mesh, build_curved_ledge_mesh, build_fluted_shaft_mesh};
use super::{
    ArenaPalette, CHAMBER_HEIGHT, LEVEL_RADII, LEVEL_Y, LEVEL1_LEDGE_YS, SHAFT_HEIGHT,
    SHAFT_RADIUS, SuperFlicksElevator,
};

/// Spawns the moving circular elevator platform at Level 1 (`Y = 0.0`).
pub(super) fn spawn_elevator_platform(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    palette: &ArenaPalette,
) {
    let parent = commands
        .spawn((
            Transform::from_xyz(0.0, LEVEL_Y[0], 0.0),
            Visibility::default(),
            SuperFlicksElevator,
            ArenaEntity,
        ))
        .id();

    // Brushed metallic top deck (flush with Y = 0)
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(2.22, 0.24))),
        MeshMaterial3d(palette.elevator_deck.clone()),
        Transform::from_xyz(0.0, -0.12, 0.0),
        ChildOf(parent),
    ));

    // Subtle dark inner deck accent ring
    commands.spawn((
        Mesh3d(meshes.add(build_annulus_mesh(1.90, 2.00, 64, true))),
        MeshMaterial3d(palette.charcoal.clone()),
        Transform::from_xyz(0.0, 0.004, 0.0),
        ChildOf(parent),
    ));

    // Bright cyan-blue glowing perimeter ring around the elevator deck
    commands.spawn((
        Mesh3d(meshes.add(build_annulus_mesh(2.14, 2.38, 64, true))),
        MeshMaterial3d(palette.cyan_glow.clone()),
        Transform::from_xyz(0.0, 0.008, 0.0),
        ChildOf(parent),
    ));

    // Dark charcoal lower chassis cylinder beneath the deck
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(2.38, 0.70))),
        MeshMaterial3d(palette.charcoal.clone()),
        Transform::from_xyz(0.0, -0.37, 0.0),
        ChildOf(parent),
    ));
}

/// Spawns Level 1: cylindrical chamber plus bottom pit well, base dark curb, and 4 curved ledges.
pub(super) fn spawn_level_1(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    palette: &ArenaPalette,
) {
    let radius = LEVEL_RADII[0];
    spawn_clean_chamber(commands, meshes, palette, LEVEL_Y[0], radius);

    // Sealed elevator pit well beneath Level 1 so no void shows around platform rim
    commands.spawn((
        Mesh3d(meshes.add(build_inward_cylinder_mesh(SHAFT_RADIUS, 2.2, 48))),
        MeshMaterial3d(palette.charcoal.clone()),
        Transform::from_xyz(0.0, -2.2, 0.0),
        ArenaEntity,
    ));
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(SHAFT_RADIUS + 0.1))),
        MeshMaterial3d(palette.charcoal.clone()),
        Transform::from_xyz(0.0, -2.2, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ArenaEntity,
    ));

    // Full-360 lower architectural dark step/curb framing the bottom of Level 1
    commands.spawn((
        Mesh3d(meshes.add(build_curved_ledge_mesh(
            radius - 1.05,
            radius + 0.05,
            0.55,
            0.0,
            TAU,
            72,
        ))),
        MeshMaterial3d(palette.charcoal.clone()),
        Transform::from_xyz(0.0, 0.55, 0.0),
        ArenaEntity,
    ));

    // 4 elevated stacked curved dark charcoal catwalk ledges (indices 1..=4 of LEVEL1_LEDGE_YS)
    let ledge_half_degs: [f32; 4] = [135.0, 125.0, 110.0, 78.0];
    for (&ledge_y, &half_deg) in LEVEL1_LEDGE_YS[1..].iter().zip(ledge_half_degs.iter()) {
        let half_rad = half_deg.to_radians();
        commands.spawn((
            Mesh3d(meshes.add(build_curved_ledge_mesh(
                radius - 1.15,
                radius + 0.05,
                0.24,
                -half_rad,
                half_rad,
                56,
            ))),
            MeshMaterial3d(palette.charcoal.clone()),
            Transform::from_xyz(0.0, ledge_y, 0.0),
            ArenaEntity,
        ));
    }
}

/// Spawns a clean slate cylindrical chamber (used for Level 1 base, Level 2, and Level 3).
pub(super) fn spawn_clean_chamber(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    palette: &ArenaPalette,
    floor_y: f32,
    radius: f32,
) {
    let ceiling_y = floor_y + CHAMBER_HEIGHT;

    // Outer inward-facing slate cylindrical wall
    commands.spawn((
        Mesh3d(meshes.add(build_inward_cylinder_mesh(radius, CHAMBER_HEIGHT, 72))),
        MeshMaterial3d(palette.slate_wall.clone()),
        Transform::from_xyz(0.0, floor_y, 0.0),
        ArenaEntity,
    ));

    // Floor: cyan shaft exit rim -> dark collar -> slate floor -> outer dark reference ring
    let floor_rings: [(f32, f32, f32, u32, &Handle<StandardMaterial>); 4] = [
        (SHAFT_RADIUS, SHAFT_RADIUS + 0.16, 0.012, 64, &palette.cyan_glow),
        (SHAFT_RADIUS + 0.16, SHAFT_RADIUS + 0.65, 0.008, 64, &palette.charcoal),
        (SHAFT_RADIUS + 0.65, radius + 0.05, 0.0, 72, &palette.slate_floor),
        (4.85, 5.30, 0.010, 64, &palette.charcoal),
    ];
    for (inner_r, outer_r, y_off, segs, mat) in floor_rings {
        commands.spawn((
            Mesh3d(meshes.add(build_annulus_mesh(inner_r, outer_r, segs, true))),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(0.0, floor_y + y_off, 0.0),
            ArenaEntity,
        ));
    }

    // Outer perimeter baseboard dark trim ring
    commands.spawn((
        Mesh3d(meshes.add(build_curved_ledge_mesh(
            radius - 0.16,
            radius + 0.05,
            0.22,
            0.0,
            TAU,
            72,
        ))),
        MeshMaterial3d(palette.charcoal.clone()),
        Transform::from_xyz(0.0, floor_y + 0.22, 0.0),
        ArenaEntity,
    ));

    // Ceiling & upper shaft entrance ring + front curved overhead gantry pod
    spawn_ceiling_and_pod(commands, meshes, palette, ceiling_y, radius);
}

fn spawn_ceiling_and_pod(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    palette: &ArenaPalette,
    ceiling_y: f32,
    radius: f32,
) {
    // Main slate ceiling annulus facing downward
    commands.spawn((
        Mesh3d(meshes.add(build_annulus_mesh(SHAFT_RADIUS, radius + 0.05, 72, false))),
        MeshMaterial3d(palette.slate_ceiling.clone()),
        Transform::from_xyz(0.0, ceiling_y, 0.0),
        ArenaEntity,
    ));

    // Dark charcoal recessed ceiling ring framing the central shaft opening
    commands.spawn((
        Mesh3d(meshes.add(build_curved_ledge_mesh(
            SHAFT_RADIUS,
            SHAFT_RADIUS + 0.85,
            0.30,
            0.0,
            TAU,
            64,
        ))),
        MeshMaterial3d(palette.charcoal.clone()),
        Transform::from_xyz(0.0, ceiling_y, 0.0),
        ArenaEntity,
    ));

    // Glowing cyan-blue rim around the shaft entrance on the underside
    commands.spawn((
        Mesh3d(meshes.add(build_annulus_mesh(SHAFT_RADIUS, SHAFT_RADIUS + 0.14, 64, false))),
        MeshMaterial3d(palette.cyan_glow.clone()),
        Transform::from_xyz(0.0, ceiling_y - 0.305, 0.0),
        ArenaEntity,
    ));

    // Front-mounted curved architectural overhead gantry pod (facing -Z)
    let outer_arc = 32.0f32.to_radians();
    commands.spawn((
        Mesh3d(meshes.add(build_curved_ledge_mesh(
            SHAFT_RADIUS + 0.65,
            SHAFT_RADIUS + 1.40,
            0.26,
            -outer_arc,
            outer_arc,
            24,
        ))),
        MeshMaterial3d(palette.charcoal.clone()),
        Transform::from_xyz(0.0, ceiling_y - 0.10, 0.0),
        ArenaEntity,
    ));

    let body_arc = 25.0f32.to_radians();
    commands.spawn((
        Mesh3d(meshes.add(build_curved_ledge_mesh(
            SHAFT_RADIUS + 0.74,
            SHAFT_RADIUS + 1.28,
            0.54,
            -body_arc,
            body_arc,
            20,
        ))),
        MeshMaterial3d(palette.charcoal.clone()),
        Transform::from_xyz(0.0, ceiling_y - 0.36, 0.0),
        ArenaEntity,
    ));

    // Sleek cyan LED accent strip along the pod's upper front edge
    let strip_arc = 22.0f32.to_radians();
    commands.spawn((
        Mesh3d(meshes.add(build_curved_ledge_mesh(
            SHAFT_RADIUS + 0.62,
            SHAFT_RADIUS + 0.72,
            0.09,
            -strip_arc,
            strip_arc,
            18,
        ))),
        MeshMaterial3d(palette.cyan_glow.clone()),
        Transform::from_xyz(0.0, ceiling_y - 0.20, 0.0),
        ArenaEntity,
    ));
}

/// Spawns a vertically fluted metallic elevator shaft connecting two adjacent chambers.
pub(super) fn spawn_shaft(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    palette: &ArenaPalette,
    y_start: f32,
) {
    commands.spawn((
        Mesh3d(meshes.add(build_fluted_shaft_mesh(SHAFT_RADIUS, SHAFT_HEIGHT, 96, 0.045))),
        MeshMaterial3d(palette.shaft_wall.clone()),
        Transform::from_xyz(0.0, y_start, 0.0),
        ArenaEntity,
    ));

    // Subtle dark horizontal structural seam rings inside the shaft for motion cues
    let seam_mesh = meshes.add(build_curved_ledge_mesh(
        SHAFT_RADIUS - 0.03,
        SHAFT_RADIUS + 0.04,
        0.08,
        0.0,
        TAU,
        64,
    ));
    for step in 1..4 {
        let y = y_start + (step as f32) * (SHAFT_HEIGHT / 4.0);
        commands.spawn((
            Mesh3d(seam_mesh.clone()),
            MeshMaterial3d(palette.charcoal.clone()),
            Transform::from_xyz(0.0, y, 0.0),
            ArenaEntity,
        ));
    }
}
