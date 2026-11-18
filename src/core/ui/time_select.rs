use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::core::round::RoundDuration;
use crate::core::state::AppState;

const PRESETS: [u32; 5] = [30, 60, 120, 180, 300];

pub fn time_select_ui(
    mut contexts: EguiContexts,
    mut duration: ResMut<RoundDuration>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Round duration")
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            for seconds in PRESETS {
                let label = format!("{}:{:02}", seconds / 60, seconds % 60);
                if ui
                    .add(egui::Button::new(egui::RichText::new(label).size(20.0))
                        .min_size(egui::vec2(160.0, 40.0)))
                    .clicked()
                {
                    duration.0 = seconds;
                    next_state.set(AppState::Countdown);
                }
                ui.add_space(6.0);
            }

            ui.separator();
            if ui.button("Back").clicked() {
                next_state.set(AppState::TaskSelect);
            }
        });
}
