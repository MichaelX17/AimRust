//! Tracking Follow: one smoothly moving target; core scores time on target.
//! Follows the standard AimRust Default arena (no backboard wall, open map).

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::core::camera::PlayerCamera;
use crate::core::hit_zone::ReactionTimer;
use crate::core::input::PlayerLook;
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, TaskInfo, task_playing, task_selected};
use crate::props::targets::{TargetMaterial, spawn_sphere};

pub const INFO: TaskInfo = TaskInfo {
    id: "tracking_follow",
    name: "Tracking: Follow",
    description: "Keep the crosshair on a moving target",
    scoring: ScoringMode::Tracking,
    is_sniper_mode: false,
};

const RADIUS: f32 = 0.35;
const PLANE_Z: f32 = -15.0;
const SPEED: f32 = 3.0;
const MIN_X: f32 = -5.0;
const MAX_X: f32 = 5.0;
const MIN_Y: f32 = 0.8;
const MAX_Y: f32 = 4.0;
const MIN_CHANGE: f32 = 0.5;
const MAX_CHANGE: f32 = 1.5;
const STEER_RATE: f32 = 6.0;

#[derive(Component)]
struct TrackingFollowEntity;

#[derive(Component)]
struct TrackingFollow {
    velocity: Vec2,
    target_velocity: Vec2,
    change_in: f32,
}

#[derive(Resource)]
struct TrackingFollowRng(u64);

impl Default for TrackingFollowRng {
    fn default() -> Self {
        Self(0x94D0_49BB_1331_11EB)
    }
}

impl TrackingFollowRng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn gen_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }

    fn next_direction(&mut self) -> Vec2 {
        let angle = self.gen_f32() * TAU;
        Vec2::new(angle.cos(), angle.sin())
    }

    fn gen_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.gen_f32() * (max - min)
    }
}

pub fn plugin() -> impl Plugin {
    TrackingFollowPlugin
}

struct TrackingFollowPlugin;

impl Plugin for TrackingFollowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrackingFollowRng>()
            .add_systems(
                OnEnter(AppState::Countdown),
                setup.run_if(task_selected(INFO.id)),
            )
            .add_systems(Update, movement.run_if(task_playing(INFO.id)))
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
    mut rng: ResMut<TrackingFollowRng>,
    mut look: ResMut<PlayerLook>,
    mut cameras: Query<&mut Transform, With<PlayerCamera>>,
) {
    *look = PlayerLook::default();
    if let Ok(mut cam_t) = cameras.single_mut() {
        cam_t.translation = Vec3::new(0.0, 1.7, 0.0);
        cam_t.rotation = Quat::IDENTITY;
    }

    let start = Vec3::new(
        rng.gen_range(MIN_X, MAX_X),
        rng.gen_range(MIN_Y, MAX_Y),
        PLANE_Z,
    );
    let direction = rng.next_direction() * SPEED;
    let change_in = rng.gen_range(MIN_CHANGE, MAX_CHANGE);

    let entity = spawn_sphere(&mut commands, &mut meshes, &material, start, RADIUS, None);
    commands.entity(entity).insert((
        TrackingFollowEntity,
        ReactionTimer::new(),
        TrackingFollow {
            velocity: direction,
            target_velocity: direction,
            change_in,
        },
    ));
}

fn movement(
    time: Res<Time>,
    mut rng: ResMut<TrackingFollowRng>,
    mut targets: Query<(&mut Transform, &mut TrackingFollow), With<TrackingFollowEntity>>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut follow) in &mut targets {
        follow.change_in -= dt;
        if follow.change_in <= 0.0 {
            follow.target_velocity = rng.next_direction() * SPEED;
            follow.change_in = rng.gen_range(MIN_CHANGE, MAX_CHANGE);
        }

        let alpha = (dt * STEER_RATE).min(1.0);
        follow.velocity = follow.velocity.lerp(follow.target_velocity, alpha);

        let mut position = transform.translation;
        position.x += follow.velocity.x * dt;
        position.y += follow.velocity.y * dt;

        if position.x < MIN_X {
            position.x = MIN_X;
            follow.velocity.x = follow.velocity.x.abs();
            follow.target_velocity.x = follow.target_velocity.x.abs();
        } else if position.x > MAX_X {
            position.x = MAX_X;
            follow.velocity.x = -follow.velocity.x.abs();
            follow.target_velocity.x = -follow.target_velocity.x.abs();
        }

        if position.y < MIN_Y {
            position.y = MIN_Y;
            follow.velocity.y = follow.velocity.y.abs();
            follow.target_velocity.y = follow.target_velocity.y.abs();
        } else if position.y > MAX_Y {
            position.y = MAX_Y;
            follow.velocity.y = -follow.velocity.y.abs();
            follow.target_velocity.y = -follow.target_velocity.y.abs();
        }

        position.z = PLANE_Z;
        transform.translation = position;
    }
}

fn cleanup(mut commands: Commands, targets: Query<Entity, With<TrackingFollowEntity>>) {
    for entity in &targets {
        commands.entity(entity).despawn();
    }
}
