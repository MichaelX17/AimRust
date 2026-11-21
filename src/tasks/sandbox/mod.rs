use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::core::config::Settings;
use crate::core::hit::{Expired, Miss, Target, TargetExpired, TargetHit};
use crate::core::input::CursorFree;
use crate::core::state::AppState;
use crate::core::task_api::{
    SelectedTask, ScoringMode, TaskInfo, task_playing, task_selected,
};
use crate::core::ui::settings::settings_panel;
use crate::props::targets::{TargetMaterial, spawn_box, spawn_sphere};

pub const INFO: TaskInfo = TaskInfo {
    id: "sandbox",
    name: "Sandbox",
    description: "Test level for settings and hit detection",
    scoring: ScoringMode::Click,
    is_sniper_mode: true,
};

const LIFETIME_SECS: f32 = 3.0;

#[derive(Component)]
struct SandboxEntity;

#[derive(Component)]
struct SandboxOscillating;

#[derive(Resource, Default)]
struct SandboxPanel {
    open: bool,
}

#[derive(Resource)]
struct SandboxLastEvent(&'static str);

impl Default for SandboxLastEvent {
    fn default() -> Self {
        Self("-")
    }
}

#[derive(Resource)]
struct SandboxRng(u64);

impl Default for SandboxRng {
    fn default() -> Self {
        Self(0x9E37_79B9_7F4A_7C15)
    }
}

impl SandboxRng {
    fn next_f32(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        ((x >> 40) as u32) as f32 / 16_777_216.0
    }
}

pub fn plugin() -> impl Plugin {
    SandboxPlugin
}

struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        }

        app.init_resource::<SandboxPanel>()
            .init_resource::<SandboxLastEvent>()
            .init_resource::<SandboxRng>()
            .add_systems(Startup, ensure_selected)
            .add_systems(
                OnEnter(AppState::Countdown),
                setup.run_if(task_selected(INFO.id)),
            )
            .add_systems(OnEnter(AppState::Results), cleanup)
            .add_systems(OnEnter(AppState::Menu), cleanup)
            .add_systems(
                Update,
                (
                    oscillate,
                    handle_hits,
                    handle_expired,
                    track_last_event,
                    toggle_panel,
                )
                    .run_if(task_playing(INFO.id)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (debug_overlay, settings_window)
                    .chain()
                    .run_if(task_playing(INFO.id)),
            );
    }
}

/// Without a task select screen yet, default to sandbox so F2 debug works.
fn ensure_selected(mut selected: ResMut<SelectedTask>) {
    if selected.0.is_empty() {
        selected.0 = INFO.id;
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<TargetMaterial>,
) {
    for (pos, radius) in [
        (Vec3::new(-3.0, 1.2, -8.0), 0.15),
        (Vec3::new(0.0, 2.4, -14.0), 0.3),
        (Vec3::new(3.0, 1.0, -20.0), 0.5),
    ] {
        let entity = spawn_sphere(&mut commands, &mut meshes, &material, pos, radius, None);
        commands.entity(entity).insert(SandboxEntity);
    }

    for pos in [Vec3::new(1.8, 1.8, -10.0), Vec3::new(-2.2, 2.2, -18.0)] {
        let entity = spawn_box(
            &mut commands,
            &mut meshes,
            &material,
            pos,
            Vec3::splat(0.3),
            None,
        );
        commands.entity(entity).insert(SandboxEntity);
    }

    let oscillator = spawn_sphere(
        &mut commands,
        &mut meshes,
        &material,
        Vec3::new(0.0, 1.7, -12.0),
        0.3,
        None,
    );
    commands
        .entity(oscillator)
        .insert((SandboxEntity, SandboxOscillating));

    let expiring = spawn_sphere(
        &mut commands,
        &mut meshes,
        &material,
        Vec3::new(-2.0, 1.5, -15.0),
        0.3,
        Some(LIFETIME_SECS),
    );
    commands.entity(expiring).insert(SandboxEntity);
}

fn cleanup(
    mut commands: Commands,
    entities: Query<Entity, With<SandboxEntity>>,
    mut panel: ResMut<SandboxPanel>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    panel.open = false;
}

fn oscillate(time: Res<Time>, mut targets: Query<&mut Transform, With<SandboxOscillating>>) {
    let x = (time.elapsed_secs() * 2.0).sin() * 4.0;
    for mut transform in &mut targets {
        transform.translation.x = x;
    }
}

fn random_position(transform: &mut Transform, rng: &mut SandboxRng) {
    // Do not respawn in the same spot that was just vacated; try a few times
    // and fall back to the farthest candidate.
    let old = transform.translation;
    let mut fallback = old;
    let mut best_distance = -1.0;
    for _ in 0..8 {
        let candidate = Vec3::new(
            -6.0 + rng.next_f32() * 12.0,
            0.8 + rng.next_f32() * 3.2,
            old.z,
        );
        let distance = candidate.distance(old);
        if distance > 1.0 {
            transform.translation = candidate;
            return;
        }
        if distance > best_distance {
            best_distance = distance;
            fallback = candidate;
        }
    }
    transform.translation = fallback;
}

fn handle_hits(
    mut hits: MessageReader<TargetHit>,
    mut rng: ResMut<SandboxRng>,
    mut transforms: Query<&mut Transform, With<Target>>,
) {
    for hit in hits.read() {
        if let Ok(mut transform) = transforms.get_mut(hit.entity) {
            random_position(&mut transform, &mut rng);
        }
    }
}

fn handle_expired(
    mut expired: MessageReader<TargetExpired>,
    mut commands: Commands,
    mut rng: ResMut<SandboxRng>,
    mut targets: Query<(&mut Target, &mut Transform)>,
) {
    for event in expired.read() {
        if let Ok((mut target, mut transform)) = targets.get_mut(event.entity) {
            target.lifetime = Some(LIFETIME_SECS);
            random_position(&mut transform, &mut rng);
            commands.entity(event.entity).remove::<Expired>();
        }
    }
}

fn track_last_event(
    mut hits: MessageReader<TargetHit>,
    mut misses: MessageReader<Miss>,
    mut expired: MessageReader<TargetExpired>,
    mut last: ResMut<SandboxLastEvent>,
) {
    for _ in hits.read() {
        last.0 = "Hit";
    }
    for _ in misses.read() {
        last.0 = "Miss";
    }
    for _ in expired.read() {
        last.0 = "Expired";
    }
}

fn toggle_panel(
    keys: Res<ButtonInput<KeyCode>>,
    mut panel: ResMut<SandboxPanel>,
    mut cursor_free: ResMut<CursorFree>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !keys.just_pressed(KeyCode::F1) {
        return;
    }
    panel.open = !panel.open;
    cursor_free.0 = panel.open;

    for mut cursor in &mut cursors {
        cursor.visible = panel.open;
        cursor.grab_mode = if panel.open {
            CursorGrabMode::None
        } else {
            CursorGrabMode::Locked
        };
    }
}

fn debug_overlay(
    mut contexts: EguiContexts,
    settings: Res<Settings>,
    last: Res<SandboxLastEvent>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|diagnostic| diagnostic.smoothed())
        .unwrap_or(0.0);

    egui::Area::new(egui::Id::new("sandbox_debug"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 8.0))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.label(format!("FPS {fps:.0}"));
                ui.label(format!("sens {:.2}", settings.sens));
                ui.label(format!("fov_h {:.1}", settings.fov_h));
                ui.label(format!("last: {}", last.0));
            });
        });
}

fn settings_window(
    mut contexts: EguiContexts,
    mut settings: ResMut<Settings>,
    panel: Res<SandboxPanel>,
) {
    if !panel.open {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let max_height = (ctx.content_rect().height() - 96.0).max(240.0);
    let mut draft = settings.clone();
    egui::Window::new("Sandbox settings")
        .resizable(true)
        .collapsible(false)
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 8.0))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .max_height(max_height)
                .show(ui, |ui| settings_panel(ui, &mut draft));
        });

    if draft != *settings {
        *settings = draft;
    }
}
