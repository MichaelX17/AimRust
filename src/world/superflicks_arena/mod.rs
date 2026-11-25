//! SuperFlicks 4-story vertical elevator tower arena:
//! - Level 1 (Y =  0m): Cylindrical slate chamber with 4 stacked curved dark catwalk ledges
//! - Level 2 (Y = 20m): Clean minimalist slate cylindrical flicking chamber
//! - Level 3 (Y = 40m): Wide minimalist slate cylindrical precision chamber
//! - Level 4 (Y = 60m): Sunlit octagonal white atrium with matte teal floor, 12 perimeter
//!   black/cyan bollards, giant circular skylight oculus, and turquoise cloud skybox
//! - Connected by 3 vertical fluted metallic elevator shafts and a moving central circular
//!   elevator platform with glowing cyan rim lighting.

mod atrium;
mod chambers;
pub mod meshes;

use bevy::prelude::*;

use crate::core::arena::ArenaEntity;

/// Floor deck Y coordinates for Levels 1..=4.
pub const LEVEL_Y: [f32; 4] = [0.0, 20.0, 40.0, 60.0];

/// Interior height of Levels 1..=3.
pub const CHAMBER_HEIGHT: f32 = 14.0;

/// Interior height of Level 4 sunlit atrium up to the skylight ceiling.
pub const ATRIUM_HEIGHT: f32 = 13.5;

/// Vertical shaft height between one chamber's ceiling and the next floor's deck.
pub const SHAFT_HEIGHT: f32 = 6.0;

/// Inner bore radius of the vertical elevator shafts.
pub const SHAFT_RADIUS: f32 = 2.55;

/// Outer wall radii for Levels 1..=4.
pub const LEVEL_RADII: [f32; 4] = [13.5, 13.5, 14.5, 16.5];

/// Standing-surface Y coordinates for all 5 Level 1 lines:
/// index 0 is the lower base curb (`0.55m`), indices 1..=4 are the 4 curved catwalk ledges.
pub const LEVEL1_LEDGE_YS: [f32; 5] = [0.55, 2.75, 4.95, 7.15, 9.35];

/// Radial center of Level 1's catwalk ledges (for placing standing targets).
pub const LEVEL1_LEDGE_RADIUS: f32 = 12.95;

/// Number of perimeter floor launcher turrets on Level 4.
pub const ATRIUM_TURRET_COUNT: usize = 12;

/// Ring radius of the 12 floor launcher turrets on Level 4.
pub const ATRIUM_TURRET_RING_RADIUS: f32 = 11.2;

/// World position of the launch muzzle at the top of Level 4 turret `index`.
pub fn atrium_turret_top(index: usize) -> Vec3 {
    let idx = index % ATRIUM_TURRET_COUNT;
    let angle = (idx as f32 + 0.5) / ATRIUM_TURRET_COUNT as f32 * std::f32::consts::TAU;
    Vec3::new(
        ATRIUM_TURRET_RING_RADIUS * angle.sin(),
        LEVEL_Y[3] + 1.46,
        -ATRIUM_TURRET_RING_RADIUS * angle.cos(),
    )
}

/// Marker component for the moving central circular elevator platform.
/// Move this entity's `Transform.translation.y` alongside `PlayerCamera` (`y + 1.7`).
#[derive(Component)]
pub struct SuperFlicksElevator;

pub(super) struct ArenaPalette {
    pub slate_wall: Handle<StandardMaterial>,
    pub slate_floor: Handle<StandardMaterial>,
    pub slate_ceiling: Handle<StandardMaterial>,
    pub shaft_wall: Handle<StandardMaterial>,
    pub charcoal: Handle<StandardMaterial>,
    pub cyan_glow: Handle<StandardMaterial>,
    pub elevator_deck: Handle<StandardMaterial>,
    pub atrium_wall: Handle<StandardMaterial>,
    pub atrium_ceiling: Handle<StandardMaterial>,
    pub atrium_teal: Handle<StandardMaterial>,
    pub atrium_silver: Handle<StandardMaterial>,
    pub bollard_accent: Handle<StandardMaterial>,
    pub cloud_white: Handle<StandardMaterial>,
}

impl ArenaPalette {
    fn create(materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            slate_wall: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(0x6E, 0x79, 0x87),
                perceptual_roughness: 0.88,
                metallic: 0.04,
                reflectance: 0.25,
                ..default()
            }),
            slate_floor: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(0x5E, 0x69, 0x76),
                perceptual_roughness: 0.90,
                metallic: 0.04,
                reflectance: 0.20,
                ..default()
            }),
            slate_ceiling: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(0x58, 0x62, 0x70),
                perceptual_roughness: 0.92,
                metallic: 0.02,
                ..default()
            }),
            shaft_wall: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(0x7A, 0x85, 0x94),
                perceptual_roughness: 0.62,
                metallic: 0.18,
                reflectance: 0.45,
                ..default()
            }),
            charcoal: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(0x1B, 0x20, 0x27),
                perceptual_roughness: 0.72,
                metallic: 0.14,
                reflectance: 0.35,
                ..default()
            }),
            cyan_glow: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(0x00, 0xB8, 0xFC),
                emissive: LinearRgba::rgb(0.0, 2.0, 3.4),
                perceptual_roughness: 0.20,
                metallic: 0.05,
                ..default()
            }),
            elevator_deck: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(0xB4, 0xBE, 0xC8),
                perceptual_roughness: 0.46,
                metallic: 0.28,
                reflectance: 0.50,
                ..default()
            }),
            atrium_wall: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(0xDE, 0xE5, 0xEC),
                perceptual_roughness: 0.90,
                metallic: 0.0,
                reflectance: 0.22,
                ..default()
            }),
            atrium_ceiling: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(0xBA, 0xC3, 0xCE),
                perceptual_roughness: 0.92,
                metallic: 0.0,
                ..default()
            }),
            atrium_teal: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(0x28, 0xA4, 0x98),
                perceptual_roughness: 0.88,
                metallic: 0.02,
                reflectance: 0.18,
                ..default()
            }),
            atrium_silver: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(0xDC, 0xE2, 0xE8),
                perceptual_roughness: 0.55,
                metallic: 0.15,
                ..default()
            }),
            bollard_accent: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(0xF2, 0xF6, 0xFA),
                emissive: LinearRgba::rgb(0.25, 0.28, 0.32),
                perceptual_roughness: 0.4,
                ..default()
            }),
            cloud_white: materials.add(StandardMaterial {
                base_color: Color::srgb_u8(0xF8, 0xFE, 0xFF),
                unlit: true,
                ..default()
            }),
        }
    }
}

/// Spawns the complete 4-level SuperFlicks elevator tower arena.
pub fn spawn_superflicks_arena(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let palette = ArenaPalette::create(materials);

    // Moving central circular elevator platform (starts at Level 1)
    chambers::spawn_elevator_platform(commands, meshes, &palette);

    // Lower 3 cylindrical chambers + Level 1 catwalk ledges
    chambers::spawn_level_1(commands, meshes, &palette);
    chambers::spawn_clean_chamber(commands, meshes, &palette, LEVEL_Y[1], LEVEL_RADII[1]);
    chambers::spawn_clean_chamber(commands, meshes, &palette, LEVEL_Y[2], LEVEL_RADII[2]);

    // 3 connecting fluted vertical elevator shafts (1->2, 2->3, 3->4)
    for &floor_y in &LEVEL_Y[..3] {
        chambers::spawn_shaft(commands, meshes, &palette, floor_y + CHAMBER_HEIGHT);
    }

    // Level 4 sunlit octagonal atrium + bollards + skylight oculus + sky & clouds
    atrium::spawn_level_4_atrium(commands, meshes, materials, &palette);

    // Tower-wide lighting
    spawn_lighting(commands);
}

fn spawn_lighting(commands: &mut Commands) {
    // Angled overhead sunlight for crisp polygonal facet contrast in Level 4 & depth on ledges
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: false,
            illuminance: 18_000.0,
            color: Color::srgb(0.99, 0.99, 1.0),
            ..default()
        },
        Transform::from_xyz(14.0, 120.0, 10.0).looking_at(Vec3::new(0.0, 60.0, 0.0), Vec3::Y),
        ArenaEntity,
    ));

    // Balanced overhead fill point lights for Levels 1..=3
    let chamber_lights = [
        (LEVEL_Y[0] + 9.5, 2_000_000.0, 36.0, Color::srgb(0.93, 0.96, 1.0)),
        (LEVEL_Y[1] + 9.5, 2_000_000.0, 36.0, Color::srgb(0.93, 0.96, 1.0)),
        (LEVEL_Y[2] + 9.5, 2_200_000.0, 38.0, Color::srgb(0.93, 0.96, 1.0)),
        // Bright sunlit atrium core light for Level 4
        (LEVEL_Y[3] + 10.5, 4_800_000.0, 48.0, Color::srgb(1.0, 0.99, 0.97)),
    ];

    for (y, intensity, range, color) in chamber_lights {
        commands.spawn((
            PointLight {
                shadow_maps_enabled: false,
                intensity,
                range,
                color,
                ..default()
            },
            Transform::from_xyz(0.0, y, 0.0),
            ArenaEntity,
        ));
    }

    // Subtle cool shaft interior fill lights between floors
    for &floor_y in &LEVEL_Y[..3] {
        commands.spawn((
            PointLight {
                shadow_maps_enabled: false,
                intensity: 320_000.0,
                range: 14.0,
                color: Color::srgb(0.80, 0.94, 1.0),
                ..default()
            },
            Transform::from_xyz(0.0, floor_y + CHAMBER_HEIGHT + SHAFT_HEIGHT * 0.5, 0.0),
            ArenaEntity,
        ));
    }
}
