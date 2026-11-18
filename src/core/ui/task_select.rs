use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::core::state::AppState;
use crate::core::task_api::{SelectedTask, TaskRegistry};

pub fn task_select_ui(
    mut contexts: EguiContexts,
    registry: Res<TaskRegistry>,
    mut selected: ResMut<SelectedTask>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Select task")
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            if registry.0.is_empty() {
                ui.label("No tasks installed");
            } else {
                egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
                    for task in &registry.0 {
                        ui.horizontal(|ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(task.name).size(18.0),
                                    )
                                    .min_size(egui::vec2(220.0, 40.0)),
                                )
                                .clicked()
                            {
                                selected.0 = task.id;
                                next_state.set(AppState::TimeSelect);
                            }
                            ui.label(egui::RichText::new(task.description).weak());
                        });
                        ui.add_space(4.0);
                    }
                });
            }

            ui.separator();
            if ui.button("Back").clicked() {
                next_state.set(AppState::Menu);
            }
        });
}
