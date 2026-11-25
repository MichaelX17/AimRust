use bevy_egui::egui::{self, Align, Layout};

use crate::core::config::Settings;
use crate::core::ui::widgets;
use super::components::{self, ROW_HEIGHT, SettingsCategory};

pub fn render_environment_card(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    settings: &mut Settings,
) {
    components::settings_card(
        ui,
        width,
        height,
        SettingsCategory::Environment,
        "ENVIRONMENT",
        |ui| {
            environment_controls(ui, settings);
        },
    );
}

pub fn render_target_ui_card(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    settings: &mut Settings,
) {
    components::settings_card(
        ui,
        width,
        height,
        SettingsCategory::TargetUi,
        "TARGET & UI",
        |ui| {
            target_ui_controls(ui, settings);
        },
    );
}

pub fn environment_controls(ui: &mut egui::Ui, settings: &mut Settings) {
    color_row(ui, "Floor", &mut settings.colors.floor);
    color_row(ui, "Ceiling", &mut settings.colors.ceiling);
    color_row(ui, "Walls", &mut settings.colors.walls);
}

pub fn target_ui_controls(ui: &mut egui::Ui, settings: &mut Settings) {
    color_row(ui, "Target color", &mut settings.colors.target);
    color_row(ui, "UI accent", &mut settings.colors.background_accent);
}

fn color_row(ui: &mut egui::Ui, label: &str, color: &mut [u8; 3]) {
    let row_w = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(row_w, ROW_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(label)
                        .size(13.0)
                        .color(widgets::TEXT_WEAK),
                )
                .truncate(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                components::color_swatch(ui, color, 56.0, 22.0);
            });
        },
    );
}
