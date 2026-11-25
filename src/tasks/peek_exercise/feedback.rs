use bevy::prelude::*;

use crate::core::hit::{CoverWall, Miss, TargetHit};
use crate::core::hit_zone::HitZone;
use crate::core::score::RoundScore;
use super::types::*;

pub fn handle_shots(
    mut hits: MessageReader<TargetHit>,
    mut misses: MessageReader<Miss>,
    state: Res<PeekExerciseState>,
    mut score: ResMut<RoundScore>,
    mut targets: Query<&mut PeekTarget>,
) {
    for hit in hits.read() {
        if hit.hit_zone == Some(HitZone::Head)
            && let Some(entity) = state.active_target
            && let Ok(mut target) = targets.get_mut(entity)
            && !matches!(target.phase, PeekPhase::Feedback { .. } | PeekPhase::Done)
        {
            score.points += 10;
            score.hits += 1;
            target.phase = PeekPhase::Feedback {
                timer: Timer::from_seconds(0.3, TimerMode::Once),
                is_headshot: true,
            };
        } else if hit.hit_zone == Some(HitZone::Body)
            && let Some(entity) = state.active_target
            && let Ok(target) = targets.get(entity)
            && !matches!(target.phase, PeekPhase::Feedback { .. } | PeekPhase::Done)
        {
            score.points -= 2;
        }
    }

    for _ in misses.read() {
        score.points -= 2;
        score.misses += 1;
    }
}

pub fn sync_target_mesh_material(
    peek_materials: Res<PeekMaterials>,
    targets: Query<&PeekTarget>,
    mut meshes: Query<(Entity, &mut MeshMaterial3d<StandardMaterial>), Without<CoverWall>>,
    parents: Query<&ChildOf>,
) {
    for (entity, mut mat_handle) in &mut meshes {
        let mut curr = entity;
        while let Ok(child_of) = parents.get(curr) {
            let parent = child_of.parent();
            if let Ok(target) = targets.get(parent) {
                let desired = match target.phase {
                    PeekPhase::Feedback {
                        is_headshot: true, ..
                    } => &peek_materials.headshot,
                    PeekPhase::Feedback {
                        is_headshot: false, ..
                    } => &peek_materials.despawn,
                    _ => &peek_materials.target,
                };
                if mat_handle.0 != *desired {
                    *mat_handle = MeshMaterial3d(desired.clone());
                }
                break;
            }
            curr = parent;
        }
    }
}
