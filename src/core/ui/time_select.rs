use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::core::round::RoundDuration;
use crate::core::settings::UserSettings;
use crate::core::state::AppState;

const PRESETS: [u32; 5] = [30, 60, 120, 180, 300];

#[derive(Resource, Default)]
pub struct DoaPromptState {
    pub open: bool,
    pub pending_duration: u32,
}

pub fn time_select_ui(
    mut contexts: EguiContexts,
    mut duration: ResMut<RoundDuration>,
    mut settings: ResMut<UserSettings>,
    mut prompt_state: Local<DoaPromptState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    if prompt_state.open {
        egui::Window::new("Dead or Alive Mode")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("Enable Dead or Alive mode?\n(One missed shot ends the round)")
                        .size(16.0),
                );
                ui.add_space(14.0);

                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Yes (Dead or Alive)")
                                    .color(egui::Color32::from_rgb(255, 90, 90))
                                    .size(15.0),
                            )
                            .min_size(egui::vec2(160.0, 36.0)),
                        )
                        .clicked()
                    {
                        settings.doa_mode_enabled = true;
                        duration.0 = prompt_state.pending_duration;
                        prompt_state.open = false;
                        next_state.set(AppState::Countdown);
                    }

                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("No (Normal)")
                                    .size(15.0),
                            )
                            .min_size(egui::vec2(120.0, 36.0)),
                        )
                        .clicked()
                    {
                        settings.doa_mode_enabled = false;
                        duration.0 = prompt_state.pending_duration;
                        prompt_state.open = false;
                        next_state.set(AppState::Countdown);
                    }
                });

                ui.add_space(8.0);
                if ui.button("Cancel").clicked() {
                    prompt_state.open = false;
                }
            });
        return;
    }

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
                    prompt_state.pending_duration = seconds;
                    prompt_state.open = true;
                }
                ui.add_space(6.0);
            }

            ui.separator();
            if ui.button("Back").clicked() {
                next_state.set(AppState::TaskSelect);
            }
        });
}
