use bevy::prelude::*;

use crate::core::hit::CoverWall;
use crate::core::score::RoundScore;
use crate::props::targets::spawn_capsule_target;
use super::types::*;

pub type PeekWallQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Transform,
    (With<CoverWall>, With<PeekExerciseEntity>, Without<PeekTarget>),
>;

pub fn rotate_depth(
    state: &mut PeekExerciseState,
    rng: &mut PeekRng,
    wall_query: &mut PeekWallQuery,
) {
    state.peeks_completed_in_set = 0;
    if state.remaining_depths.is_empty() {
        let mut depths = vec![WallDepth::Close, WallDepth::Mid, WallDepth::Far];
        rng.shuffle_depths(&mut depths);
        state.remaining_depths = depths;
    }
    state.current_depth = state.remaining_depths.pop().unwrap_or(WallDepth::Close);
    state.first_side_is_left = rng.gen_bool();

    let new_z = state.current_depth.z();
    for mut transform in wall_query.iter_mut() {
        transform.translation.z = new_z;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_and_move_peeks(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    peek_materials: Res<PeekMaterials>,
    mut state: ResMut<PeekExerciseState>,
    mut rng: ResMut<PeekRng>,
    mut score: ResMut<RoundScore>,
    mut targets: Query<(&mut Transform, &mut PeekTarget)>,
    mut walls: PeekWallQuery,
) {
    let dt = time.delta_secs();

    // 1. Target movement & lifecycle
    let mut target_expired = false;
    for (mut transform, mut target) in &mut targets {
        target.lifetime_timer.tick(time.delta());
        if target.lifetime_timer.is_finished()
            && !matches!(target.phase, PeekPhase::Feedback { .. } | PeekPhase::Done)
        {
            target.phase = PeekPhase::Feedback {
                timer: Timer::from_seconds(0.3, TimerMode::Once),
                is_headshot: false,
            };
            score.points -= 5;
        }

        let dir = if target.side_is_left { 1.0 } else { -1.0 };

        match target.phase {
            PeekPhase::Out => {
                if target.style == PeekStyle::JumpPeek {
                    target.jump_time += dt;
                    let t = (target.jump_time / 0.55).min(1.0);
                    transform.translation.x += dir * target.speed_out * dt;
                    transform.translation.y = (4.0 * 0.75 * t * (1.0 - t)).max(0.0);
                    if target.jump_time >= 0.55 {
                        target.jump_time = 0.55;
                        transform.translation.y = 0.0;
                        target.phase = PeekPhase::Hold(Timer::from_seconds(1.0, TimerMode::Once));
                    }
                } else {
                    transform.translation.x += dir * target.speed_out * dt;
                    let reached = if target.side_is_left {
                        transform.translation.x >= target.apex_x
                    } else {
                        transform.translation.x <= target.apex_x
                    };
                    if reached {
                        transform.translation.x = target.apex_x;
                        if target.style == PeekStyle::WideThenClose {
                            target.phase = PeekPhase::CounterStrafe;
                        } else {
                            let hold_secs = match target.style {
                                PeekStyle::CloseFast => 0.2,
                                PeekStyle::CloseSlow => 0.5,
                                PeekStyle::WideFast => 0.4,
                                PeekStyle::WideSlow => 0.5,
                                _ => 0.3,
                            };
                            target.phase =
                                PeekPhase::Hold(Timer::from_seconds(hold_secs, TimerMode::Once));
                        }
                    }
                }
            }
            PeekPhase::CounterStrafe => {
                transform.translation.x -= dir * 5.5 * dt;
                let reached = if target.side_is_left {
                    transform.translation.x <= target.counter_x
                } else {
                    transform.translation.x >= target.counter_x
                };
                if reached {
                    transform.translation.x = target.counter_x;
                    target.phase = PeekPhase::Hold(Timer::from_seconds(0.4, TimerMode::Once));
                }
            }
            PeekPhase::Hold(ref mut timer) => {
                timer.tick(time.delta());
                if timer.is_finished() {
                    if target.style == PeekStyle::JumpPeek {
                        target.phase = PeekPhase::Feedback {
                            timer: Timer::from_seconds(0.3, TimerMode::Once),
                            is_headshot: false,
                        };
                        score.points -= 5;
                    } else {
                        target.phase = PeekPhase::Retreat;
                    }
                }
            }
            PeekPhase::Retreat => {
                transform.translation.x -= dir * target.speed_retreat * dt;
                let reached_start = if target.side_is_left {
                    transform.translation.x <= target.start_x
                } else {
                    transform.translation.x >= target.start_x
                };
                if reached_start {
                    target.phase = PeekPhase::Feedback {
                        timer: Timer::from_seconds(0.3, TimerMode::Once),
                        is_headshot: false,
                    };
                    score.points -= 5;
                }
            }
            PeekPhase::Feedback { ref mut timer, .. } => {
                timer.tick(time.delta());
                if timer.is_finished() {
                    target.phase = PeekPhase::Done;
                }
            }
            PeekPhase::Done => {
                target_expired = true;
            }
        }
    }

    if target_expired {
        if let Some(entity) = state.active_target.take() {
            commands.entity(entity).despawn();
        }
        state.peeks_completed_in_set += 1;
        state.cooldown_timer = Timer::from_seconds(1.0, TimerMode::Once);

        if state.peeks_completed_in_set >= 10 {
            rotate_depth(&mut state, &mut rng, &mut walls);
        }
    }

    // 2. Spawn next peek target if none active
    if state.active_target.is_none() {
        state.cooldown_timer.tick(time.delta());
        if state.cooldown_timer.is_finished() {
            let side_is_left = if state.peeks_completed_in_set < 5 {
                state.first_side_is_left
            } else {
                !state.first_side_is_left
            };

            if state.remaining_styles.is_empty() {
                let mut styles = vec![
                    PeekStyle::CloseFast,
                    PeekStyle::CloseSlow,
                    PeekStyle::WideFast,
                    PeekStyle::WideSlow,
                    PeekStyle::WideThenClose,
                    PeekStyle::JumpPeek,
                ];
                rng.shuffle_styles(&mut styles);
                state.remaining_styles = styles;
            }
            let style = state.remaining_styles.pop().unwrap_or(PeekStyle::CloseFast);

            let wall_edge_x = if side_is_left {
                -WALL_EDGE_X
            } else {
                WALL_EDGE_X
            };
            let dir = if side_is_left { 1.0 } else { -1.0 };
            let depth_z = state.current_depth.z();
            let start_x = wall_edge_x - dir * 0.7;

            let (apex_x, counter_x, speed_out, speed_retreat) = match style {
                PeekStyle::CloseFast => (wall_edge_x + dir * 0.35, 0.0, 6.0, 6.0),
                PeekStyle::CloseSlow => (wall_edge_x + dir * 0.40, 0.0, 2.2, 3.0),
                PeekStyle::WideFast => (wall_edge_x + dir * 1.60, 0.0, 6.5, 6.0),
                PeekStyle::WideSlow => (wall_edge_x + dir * 1.50, 0.0, 2.6, 3.5),
                PeekStyle::WideThenClose => (
                    wall_edge_x + dir * 1.60,
                    wall_edge_x + dir * 0.35,
                    6.2,
                    6.0,
                ),
                PeekStyle::JumpPeek => (wall_edge_x + dir * 1.10, 0.0, 5.0, 5.5),
            };

            let target_pos = Vec3::new(start_x, 0.0, depth_z - 0.35);
            let target_entity = spawn_capsule_target(
                &mut commands,
                &mut meshes,
                peek_materials.target.clone(),
                target_pos,
            );

            commands.entity(target_entity).insert((
                PeekTarget {
                    style,
                    side_is_left,
                    wall_edge_x,
                    depth_z,
                    lifetime_timer: Timer::from_seconds(2.5, TimerMode::Once),
                    phase: PeekPhase::Out,
                    start_x,
                    apex_x,
                    counter_x,
                    speed_out,
                    speed_retreat,
                    jump_time: 0.0,
                },
                PeekExerciseEntity,
            ));

            state.active_target = Some(target_entity);
        }
    }
}


