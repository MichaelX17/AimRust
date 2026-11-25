use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::core::state::AppState;

/// Chosen round length in seconds.
#[derive(Resource)]
pub struct RoundDuration(pub u32);

impl Default for RoundDuration {
    fn default() -> Self {
        Self(60)
    }
}

/// Round clock; ticks only while playing.
#[derive(Resource, Default)]
pub struct RoundTimer {
    pub duration: f32,
    pub remaining: f32,
    pub doa_failed: bool,
}

#[derive(Resource)]
struct Countdown {
    remaining: f32,
}

impl Default for Countdown {
    fn default() -> Self {
        Self { remaining: 3.0 }
    }
}

pub struct RoundPlugin;

impl Plugin for RoundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoundDuration>()
            .init_resource::<RoundTimer>()
            .init_resource::<Countdown>()
            .add_systems(OnEnter(AppState::Countdown), start_round)
            .add_systems(
                Update,
                tick_countdown.run_if(in_state(AppState::Countdown)),
            )
            .add_systems(Update, tick_round.run_if(in_state(AppState::Playing)))
            .add_systems(
                EguiPrimaryContextPass,
                countdown_ui.run_if(in_state(AppState::Countdown)),
            );
    }
}

fn start_round(
    duration: Res<RoundDuration>,
    mut timer: ResMut<RoundTimer>,
    mut countdown: ResMut<Countdown>,
) {
    timer.duration = duration.0 as f32;
    timer.remaining = duration.0 as f32;
    timer.doa_failed = false;
    countdown.remaining = 3.0;
}

fn tick_countdown(
    time: Res<Time>,
    mut countdown: ResMut<Countdown>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    countdown.remaining -= time.delta_secs();
    if countdown.remaining <= 0.0 {
        next_state.set(AppState::Playing);
    }
}

fn tick_round(
    time: Res<Time>,
    mut timer: ResMut<RoundTimer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if timer.remaining <= 0.0 {
        return;
    }
    timer.remaining = (timer.remaining - time.delta_secs()).max(0.0);
    if timer.remaining <= 0.0 {
        next_state.set(AppState::Results);
    }
}

/// Big centered 3-2-1 text during the countdown with a drop shadow for contrast.
fn countdown_ui(mut contexts: EguiContexts, countdown: Res<Countdown>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let text = (countdown.remaining.ceil().clamp(1.0, 3.0) as u32).to_string();
    let font = egui::FontId::proportional(140.0);
    let center = ctx.content_rect().center();

    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("countdown"),
    ));

    // Soft dark drop shadow so white numbers stand out cleanly against light walls/sky
    painter.text(
        center + egui::vec2(4.0, 6.0),
        egui::Align2::CENTER_CENTER,
        &text,
        font.clone(),
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200),
    );
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        &text,
        font,
        egui::Color32::WHITE,
    );
}
