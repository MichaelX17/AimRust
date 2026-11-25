use bevy_egui::egui;

use crate::core::config::Settings;
use super::components::{self, SettingsCategory};

pub fn render_aim_card(ui: &mut egui::Ui, width: f32, height: f32, settings: &mut Settings) {
    components::settings_card(
        ui,
        width,
        height,
        SettingsCategory::Aim,
        "AIM",
        |ui| {
            aim_controls(ui, settings);
        },
    );
}

pub fn aim_controls(ui: &mut egui::Ui, settings: &mut Settings) {
    components::slider_row(
        ui,
        "Sensitivity",
        &mut settings.sens,
        0.1..=10.0,
        1,
        None,
    );
    components::slider_row(
        ui,
        "FOV (Horizontal)",
        &mut settings.fov_h,
        60.0..=130.0,
        1,
        None,
    );
}
