use bevy_egui::egui;

use crate::core::config::Settings;
use super::components::{self, SettingsCategory};

pub fn render_audio_card(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    settings: &mut Settings,
) {
    components::settings_card(
        ui,
        width,
        height,
        SettingsCategory::Audio,
        "AUDIO",
        |ui| {
            audio_controls(ui, settings);
        },
    );
}

pub fn audio_controls(ui: &mut egui::Ui, settings: &mut Settings) {
    components::toggle_row(ui, "Hit & Miss sounds", &mut settings.audio.enabled);
    ui.add_enabled_ui(settings.audio.enabled, |ui| {
        components::slider_row_u32(
            ui,
            "Volume (%)",
            &mut settings.audio.volume,
            0..=100,
            None,
        );
    });
}
