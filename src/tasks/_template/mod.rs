//! Copy this folder to `src/tasks/<name>/`, set `INFO`, then add `<name>` to
//! `register_tasks!(...)` in `src/tasks/mod.rs`.

use bevy::prelude::*;

use crate::core::hit::{Target, TargetShape};
use crate::core::hit_zone::{HitZone, ReactionTimer, TargetHitbox};
use crate::core::task_api::{ScoringMode, TaskInfo};
use crate::props::targets::TargetMaterial;

pub const INFO: TaskInfo = TaskInfo {
    id: "template",
    name: "Template",
    description: "Copy of the task module template.",
    scoring: ScoringMode::Click,
    is_sniper_mode: false,
};

pub fn plugin() -> impl Plugin {
    TemplatePlugin
}

struct TemplatePlugin;

impl Plugin for TemplatePlugin {
    fn build(&self, _app: &mut App) {
        // Round scene setup (spawn targets/props) when this task's countdown starts:
        // _app.add_systems(OnEnter(AppState::Countdown), setup.run_if(task_selected(INFO.id)));
        //
        // Gameplay systems, only while this task is playing:
        // _app.add_systems(Update, (tick, respawn).run_if(task_playing(INFO.id)));
        //
        // Cleanup on leaving the round (both paths):
        // _app.add_systems(OnEnter(AppState::Results), cleanup);
        // _app.add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

/// Helper that spawns a humanoid reaction target with Body and Head hitboxes and ReactionTimer.
#[allow(dead_code)]
pub fn spawn_reaction_target(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &TargetMaterial,
    pos: Vec3,
) -> Entity {
    let body_radius = 0.4;
    let head_radius = 0.2;
    let head_offset = Vec3::new(0.0, 0.55, 0.0);

    let parent = commands
        .spawn((
            Transform::from_translation(pos),
            Visibility::default(),
            ReactionTimer::new(),
        ))
        .id();

    // Body entity
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(body_radius))),
        MeshMaterial3d(material.0.clone()),
        Transform::IDENTITY,
        ChildOf(parent),
        Target {
            shape: TargetShape::Sphere { radius: body_radius },
            lifetime: None,
        },
        TargetHitbox {
            zone: HitZone::Body,
        },
    ));

    // Head entity
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(head_radius))),
        MeshMaterial3d(material.0.clone()),
        Transform::from_translation(head_offset),
        ChildOf(parent),
        Target {
            shape: TargetShape::Sphere { radius: head_radius },
            lifetime: None,
        },
        TargetHitbox {
            zone: HitZone::Head,
        },
    ));

    parent
}

#[allow(dead_code)]
fn setup() {
    // Spawn this task's targets using props builders; never touch config/menus/scoring.
}

#[allow(dead_code)]
fn tick() {
    // React to TargetHit / Miss / TargetExpired, AimingAt, or move targets.
}
