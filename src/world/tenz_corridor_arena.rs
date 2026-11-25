//! VCT NA - TenZ dark neon cyber-corridor arena:
//! - Long enclosed matte obsidian-charcoal rectangular tunnel stretching down -Z
//! - Glowing crimson-rose neon perspective rails along all 4 longitudinal corners,
//!   far spawn portal rectangle, and near player defense threshold gate
//! - Procedural glowing crimson geometric X-chevron emblem centered on the far wall
//! - 3 diegetic 3D LED life indicator bars below the far emblem (green -> red on breach)
//! - Balanced corridor lighting for crisp target readability from Z = -33m to Z = 0m

use bevy::prelude::*;

use crate::core::arena::ArenaEntity;

pub const CORRIDOR_WIDTH: f32 = 9.0;
pub const CORRIDOR_HEIGHT: f32 = 4.6;
pub const CORRIDOR_Z_NEAR: f32 = 2.0;
pub const CORRIDOR_Z_FAR: f32 = -34.0;
pub const SPAWN_Z: f32 = -33.2;
pub const BREACH_Z: f32 = -0.35;

/// Diegetic 3D life indicator pip (`0..=2`, left to right) on the far wall.
#[derive(Component)]
pub struct TenZLifePip(pub usize);

/// Marks the near threshold gate bars so they can briefly flash bright crimson on breach.
#[derive(Component)]
pub struct TenZBreachGate;

/// Shared material handles for toggling the 3 life pips and flashing the breach gate.
#[derive(Resource)]
pub struct TenZIndicatorMaterials {
    pub alive: Handle<StandardMaterial>,
    pub breached: Handle<StandardMaterial>,
    pub gate_normal: Handle<StandardMaterial>,
    pub gate_flash: Handle<StandardMaterial>,
}

/// Spawns the complete VCT NA - TenZ neon-framed dark corridor arena.
pub fn spawn_tenz_corridor_arena(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let length = CORRIDOR_Z_NEAR - CORRIDOR_Z_FAR;
    let z_center = 0.5 * (CORRIDOR_Z_NEAR + CORRIDOR_Z_FAR);
    let half_w = CORRIDOR_WIDTH * 0.5;
    let half_h = CORRIDOR_HEIGHT * 0.5;

    let wall_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(0x13, 0x16, 0x1B),
        perceptual_roughness: 0.94,
        metallic: 0.04,
        reflectance: 0.12,
        ..default()
    });
    let floor_mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(0x17, 0x1A, 0x20),
        perceptual_roughness: 0.78,
        metallic: 0.08,
        reflectance: 0.25,
        ..default()
    });
    let neon_crimson = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(0xE8, 0x29, 0x52),
        emissive: LinearRgba::rgb(2.2, 0.24, 0.58),
        perceptual_roughness: 0.25,
        ..default()
    });
    let gate_flash = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(0xFF, 0x30, 0x58),
        emissive: LinearRgba::rgb(7.0, 0.5, 1.1),
        perceptual_roughness: 0.2,
        ..default()
    });
    let pip_alive = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(0x1E, 0xF5, 0x64),
        emissive: LinearRgba::rgb(0.3, 4.5, 1.0),
        perceptual_roughness: 0.2,
        ..default()
    });
    let pip_breached = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(0xFF, 0x20, 0x38),
        emissive: LinearRgba::rgb(4.8, 0.25, 0.45),
        perceptual_roughness: 0.2,
        ..default()
    });

    commands.insert_resource(TenZIndicatorMaterials {
        alive: pip_alive.clone(),
        breached: pip_breached,
        gate_normal: neon_crimson.clone(),
        gate_flash,
    });

    // 1. Enclosed tunnel shell (floor, ceiling, left/right walls, far wall, rear wall)
    let shell: [(Vec3, Vec3, &Handle<StandardMaterial>); 6] = [
        (Vec3::new(CORRIDOR_WIDTH, 0.2, length), Vec3::new(0.0, -0.1, z_center), &floor_mat),
        (
            Vec3::new(CORRIDOR_WIDTH, 0.2, length),
            Vec3::new(0.0, CORRIDOR_HEIGHT + 0.1, z_center),
            &wall_mat,
        ),
        (
            Vec3::new(0.2, CORRIDOR_HEIGHT, length),
            Vec3::new(-half_w - 0.1, half_h, z_center),
            &wall_mat,
        ),
        (
            Vec3::new(0.2, CORRIDOR_HEIGHT, length),
            Vec3::new(half_w + 0.1, half_h, z_center),
            &wall_mat,
        ),
        (
            Vec3::new(CORRIDOR_WIDTH + 0.4, CORRIDOR_HEIGHT + 0.4, 0.2),
            Vec3::new(0.0, half_h, CORRIDOR_Z_FAR - 0.1),
            &wall_mat,
        ),
        (
            Vec3::new(CORRIDOR_WIDTH + 0.4, CORRIDOR_HEIGHT + 0.4, 0.2),
            Vec3::new(0.0, half_h, CORRIDOR_Z_NEAR + 0.1),
            &wall_mat,
        ),
    ];
    for (size, pos, mat) in shell {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(mat.clone()),
            Transform::from_translation(pos),
            ArenaEntity,
        ));
    }

    // 2. Glowing crimson perspective rails, far portal frame, and near defense gate
    spawn_neon_rails(commands, meshes, &neon_crimson, length, z_center, half_w);

    // 3. Far-wall glowing crimson geometric X-chevron emblem + floor perspective chevrons
    spawn_far_emblem_and_floor_accents(commands, meshes, &neon_crimson);

    // 4. Three diegetic 3D LED life indicator bars at bottom center of the far wall
    let pip_mesh = meshes.add(Cuboid::new(0.24, 0.46, 0.08));
    for (i, x) in [-0.42f32, 0.0, 0.42].iter().enumerate() {
        commands.spawn((
            Mesh3d(pip_mesh.clone()),
            MeshMaterial3d(pip_alive.clone()),
            Transform::from_xyz(*x, 0.30, CORRIDOR_Z_FAR + 0.08),
            TenZLifePip(i),
            ArenaEntity,
        ));
    }

    // 5. Balanced corridor lighting
    spawn_corridor_lighting(commands);
}

fn spawn_neon_rails(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    neon_mat: &Handle<StandardMaterial>,
    length: f32,
    z_center: f32,
    half_w: f32,
) {
    // 4 longitudinal corner neon strips
    let long_rail = meshes.add(Cuboid::new(0.11, 0.11, length));
    for (x, y) in [
        (-half_w + 0.05, 0.05),
        (half_w - 0.05, 0.05),
        (-half_w + 0.05, CORRIDOR_HEIGHT - 0.05),
        (half_w - 0.05, CORRIDOR_HEIGHT - 0.05),
    ] {
        commands.spawn((
            Mesh3d(long_rail.clone()),
            MeshMaterial3d(neon_mat.clone()),
            Transform::from_xyz(x, y, z_center),
            ArenaEntity,
        ));
    }

    // Far portal rectangular neon frame
    let far_z = CORRIDOR_Z_FAR + 0.05;
    let horiz = meshes.add(Cuboid::new(CORRIDOR_WIDTH, 0.12, 0.10));
    let vert = meshes.add(Cuboid::new(0.12, CORRIDOR_HEIGHT, 0.10));
    for (mesh, pos) in [
        (horiz.clone(), Vec3::new(0.0, 0.06, far_z)),
        (horiz, Vec3::new(0.0, CORRIDOR_HEIGHT - 0.06, far_z)),
        (vert.clone(), Vec3::new(-half_w + 0.06, CORRIDOR_HEIGHT * 0.5, far_z)),
        (vert, Vec3::new(half_w - 0.06, CORRIDOR_HEIGHT * 0.5, far_z)),
    ] {
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(neon_mat.clone()),
            Transform::from_translation(pos),
            ArenaEntity,
        ));
    }

    // Near defense threshold gate right along BREACH_Z (flashes brighter on breach)
    let gate_vert = meshes.add(Cuboid::new(0.18, CORRIDOR_HEIGHT, 0.18));
    let gate_floor = meshes.add(Cuboid::new(CORRIDOR_WIDTH, 0.03, 0.14));
    for (mesh, pos) in [
        (gate_vert.clone(), Vec3::new(-half_w + 0.09, CORRIDOR_HEIGHT * 0.5, BREACH_Z)),
        (gate_vert, Vec3::new(half_w - 0.09, CORRIDOR_HEIGHT * 0.5, BREACH_Z)),
        (gate_floor, Vec3::new(0.0, 0.015, BREACH_Z)),
    ] {
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(neon_mat.clone()),
            Transform::from_translation(pos),
            TenZBreachGate,
            ArenaEntity,
        ));
    }
}

fn spawn_far_emblem_and_floor_accents(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    neon_mat: &Handle<StandardMaterial>,
) {
    let emblem_z = CORRIDOR_Z_FAR + 0.07;
    let cy = CORRIDOR_HEIGHT * 0.5 + 0.12;
    let arm_mesh = meshes.add(Cuboid::new(1.15, 0.24, 0.06));
    let diag_mesh = meshes.add(Cuboid::new(1.35, 0.22, 0.06));

    // Symmetric 4-way geometric V-chevron / X emblem centered on the far portal wall
    let strokes: [(Handle<Mesh>, Vec3, f32); 8] = [
        (diag_mesh.clone(), Vec3::new(-0.95, cy + 0.52, emblem_z), -0.78),
        (diag_mesh.clone(), Vec3::new(-0.95, cy - 0.52, emblem_z), 0.78),
        (diag_mesh.clone(), Vec3::new(0.95, cy + 0.52, emblem_z), 0.78),
        (diag_mesh, Vec3::new(0.95, cy - 0.52, emblem_z), -0.78),
        (arm_mesh.clone(), Vec3::new(-0.34, cy + 1.10, emblem_z), -0.78),
        (arm_mesh.clone(), Vec3::new(0.34, cy + 1.10, emblem_z), 0.78),
        (arm_mesh.clone(), Vec3::new(-0.34, cy - 1.10, emblem_z), 0.78),
        (arm_mesh, Vec3::new(0.34, cy - 1.10, emblem_z), -0.78),
    ];
    for (mesh, pos, rot_z) in strokes {
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(neon_mat.clone()),
            Transform::from_translation(pos).with_rotation(Quat::from_rotation_z(rot_z)),
            ArenaEntity,
        ));
    }

    // Sleek minimal crimson floor chevron accent down the corridor centerline
    let floor_bar = meshes.add(Cuboid::new(1.6, 0.015, 0.12));
    for z in [-14.0f32, -16.2] {
        for (x, rot_y) in [(-0.55f32, -0.45f32), (0.55, 0.45)] {
            commands.spawn((
                Mesh3d(floor_bar.clone()),
                MeshMaterial3d(neon_mat.clone()),
                Transform::from_xyz(x, 0.012, z).with_rotation(Quat::from_rotation_y(rot_y)),
                ArenaEntity,
            ));
        }
    }
}

fn spawn_corridor_lighting(commands: &mut Commands) {
    // Soft neutral directional light to give clean 3D shading on rushing sphere targets
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: false,
            illuminance: 10_000.0,
            color: Color::srgb(0.96, 0.96, 1.0),
            ..default()
        },
        Transform::from_xyz(3.0, 12.0, 6.0).looking_at(Vec3::new(0.0, 1.8, -18.0), Vec3::Y),
        ArenaEntity,
    ));

    // Even corridor ceiling fill lights so rushing targets stay crisp at every depth
    for z in [-4.0f32, -14.0, -24.0, -32.0] {
        commands.spawn((
            PointLight {
                shadow_maps_enabled: false,
                intensity: 380_000.0,
                range: 22.0,
                color: Color::srgb(0.96, 0.94, 0.98),
                ..default()
            },
            Transform::from_xyz(0.0, CORRIDOR_HEIGHT - 0.35, z),
            ArenaEntity,
        ));
    }
}
