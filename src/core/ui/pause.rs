use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::core::state::AppState;

/// Esc in Playing opens the pause menu.
pub fn esc_pause(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Paused);
    }
}

/// Esc in Paused resumes the round.
pub fn esc_resume(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Playing);
    }
}

pub fn pause_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Paused")
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.set_width(240.0);
                ui.add_space(8.0);
                if ui
                    .add(egui::Button::new(egui::RichText::new("Resume").size(20.0))
                        .min_size(egui::vec2(220.0, 44.0)))
                    .clicked()
                {
                    next_state.set(AppState::Playing);
                }
                ui.add_space(10.0);
                if ui
                    .add(egui::Button::new(egui::RichText::new("Quit").size(20.0))
                        .min_size(egui::vec2(220.0, 44.0)))
                    .clicked()
                {
                    next_state.set(AppState::Menu);
                }
                ui.add_space(8.0);
            });
        });
}
