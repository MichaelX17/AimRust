//! Target prop builders. Consumed by task modules from Task_10.
#![allow(dead_code)]

use bevy::prelude::*;

use crate::core::config::{ColorsCfg, Settings};
use crate::core::hit::{Target, TargetShape};

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
