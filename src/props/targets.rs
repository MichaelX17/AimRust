//! Target prop builders. Consumed by task modules from Task_10.
#![allow(dead_code)]

use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::core::config::{ColorsCfg, Settings};
use crate::core::hit::{Target, TargetShape};
use crate::core::hit_zone::{HitZone, ReactionTimer, TargetHitbox};

/// Shared material for all targets.
#[derive(Resource)]
pub struct TargetMaterial(pub Handle<StandardMaterial>);

fn target_surface(rgb: [u8; 3]) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgb_u8(rgb[0], rgb[1], rgb[2]),
        perceptual_roughness: 0.8,
        metallic: 0.0,
        ..default()
    }
}

/// Creates the shared target material, corrected on the next update if the
/// settings resource is not available yet.
pub fn create_target_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Option<Res<Settings>>,
) {
    let rgb = settings
        .as_ref()
        .map(|settings| settings.colors.target)
        .unwrap_or(ColorsCfg::default().target);
    commands.insert_resource(TargetMaterial(materials.add(target_surface(rgb))));
}

/// Recolors the shared target material when the target color changes.
pub fn update_target_material(
    settings: Res<Settings>,
    target_material: Option<Res<TargetMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !settings.is_changed() {
        return;
    }
    let Some(target_material) = target_material else {
        return;
    };
    let [r, g, b] = settings.colors.target;
    if let Some(mut material) = materials.get_mut(&target_material.0) {
        material.base_color = Color::srgb_u8(r, g, b);
    }
}

/// Spawns a spherical target and returns its entity.
pub fn spawn_sphere(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    pos: Vec3,
    radius: f32,
    lifetime: Option<f32>,
) -> Entity {
    commands
        .spawn((
            Mesh3d(meshes.add(Sphere::new(radius))),
            MeshMaterial3d(material.0.clone()),
            Transform::from_translation(pos),
            Target {
                shape: TargetShape::Sphere { radius },
                lifetime,
            },
        ))
        .id()
}

/// Spawns an axis-aligned box target and returns its entity.
pub fn spawn_box(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    pos: Vec3,
    half_extents: Vec3,
    lifetime: Option<f32>,
) -> Entity {
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(
                half_extents.x * 2.0,
                half_extents.y * 2.0,
                half_extents.z * 2.0,
            ))),
            MeshMaterial3d(material.0.clone()),
            Transform::from_translation(pos),
            Target {
                shape: TargetShape::Box { half_extents },
                lifetime,
            },
        ))
        .id()
}

/// Spawns a humanoid target with realistic 3D glTF model, head sphere hitbox, and torso box hitbox.
pub fn spawn_humanoid_target(
    commands: &mut Commands,
    asset_server: &AssetServer,
    pos: Vec3,
) -> Entity {
    let parent = commands
        .spawn((
            Transform::from_translation(pos),
            Visibility::default(),
            ReactionTimer::new(),
        ))
        .id();

    // 3D Humanoid visual model scaled to ~1.75m tall and facing +Z towards player
    commands.spawn((
        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/human_target.glb"))),
        Transform::from_scale(Vec3::splat(1.15))
            .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        ChildOf(parent),
    ));

    // Head collider (Sphere at ~1.55m height, radius 0.14m)
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 1.55, 0.0)),
        Target {
            shape: TargetShape::Sphere { radius: 0.14 },
            lifetime: None,
        },
        TargetHitbox {
            zone: HitZone::Head,
        },
        ChildOf(parent),
    ));

    // Torso collider (Box at ~0.85m height)
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 0.85, 0.0)),
        Target {
            shape: TargetShape::Box {
                half_extents: Vec3::new(0.20, 0.40, 0.16),
            },
            lifetime: None,
        },
        TargetHitbox {
            zone: HitZone::Body,
        },
        ChildOf(parent),
    ));

    parent
}

/// Spawns a capsule-bot target (vertical capsule body + sphere head) with head sphere
/// and body box hitboxes.
pub fn spawn_capsule_target(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    pos: Vec3,
) -> Entity {
    const BODY_RADIUS: f32 = 0.28;
    const BODY_CYL_LENGTH: f32 = 0.78;
    const BODY_HALF_HEIGHT: f32 = BODY_CYL_LENGTH * 0.5 + BODY_RADIUS; // 0.67m (total 1.34m)
    const HEAD_RADIUS: f32 = 0.16;
    const HEAD_GAP: f32 = 0.03;
    const HEAD_Y: f32 = BODY_HALF_HEIGHT * 2.0 + HEAD_GAP + HEAD_RADIUS; // 1.53m (top at 1.69m)

    let parent = commands
        .spawn((
            Transform::from_translation(pos),
            Visibility::default(),
            ReactionTimer::new(),
        ))
        .id();

    // Head sphere (visual + head hitbox)
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(HEAD_RADIUS))),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(Vec3::new(0.0, HEAD_Y, 0.0)),
        Target {
            shape: TargetShape::Sphere {
                radius: HEAD_RADIUS,
            },
            lifetime: None,
        },
        TargetHitbox {
            zone: HitZone::Head,
        },
        ChildOf(parent),
    ));

    // Capsule body (visual + body hitbox)
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(BODY_RADIUS, BODY_CYL_LENGTH))),
        MeshMaterial3d(material),
        Transform::from_translation(Vec3::new(0.0, BODY_HALF_HEIGHT, 0.0)),
        Target {
            shape: TargetShape::Box {
                half_extents: Vec3::new(BODY_RADIUS, BODY_HALF_HEIGHT, BODY_RADIUS),
            },
            lifetime: None,
        },
        TargetHitbox {
            zone: HitZone::Body,
        },
        ChildOf(parent),
    ));

    parent
}
