use bevy_egui::egui::{self, Align, Layout};

use crate::core::config::{Settings, WindowMode};
use crate::core::ui::widgets;
use super::components::{self, ROW_HEIGHT, SettingsCategory};

pub fn render_video_card(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    settings: &mut Settings,
) {
    components::settings_card(
        ui,
        width,
        height,
        SettingsCategory::Video,
        "VIDEO",
        |ui| {
            video_controls(ui, settings);
        },
    );
}

pub fn render_gameplay_card(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    settings: &mut Settings,
) {
    components::settings_card(
        ui,
        width,
        height,
        SettingsCategory::Gameplay,
        "GAMEPLAY",
        |ui| {
            gameplay_controls(ui, settings);
        },
    );
}

pub fn video_controls(ui: &mut egui::Ui, settings: &mut Settings) {
    components::toggle_row(ui, "VSync", &mut settings.video.vsync);
    components::slider_row_u32(ui, "FPS cap", &mut settings.video.fps_cap, 0..=360, None);

    let row_w = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(row_w, ROW_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.add(
                egui::Label::new(
                    egui::RichText::new("Window mode")
                        .size(13.0)
                        .color(widgets::TEXT_WEAK),
                )
                .truncate(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                egui::ComboBox::from_id_salt("window_mode")
                    .selected_text(format!("{:?}", settings.video.window_mode))
                    .width(120.0)
                    .show_ui(ui, |ui| {
                        for mode in [
                            WindowMode::Windowed,
                            WindowMode::Borderless,
                            WindowMode::Fullscreen,
                        ] {
                            ui.selectable_value(
                                &mut settings.video.window_mode,
                                mode,
                                format!("{mode:?}"),
                            );
                        }
                    });
            });
        },
    );
}

pub fn gameplay_controls(ui: &mut egui::Ui, settings: &mut Settings) {
    components::toggle_row(ui, "Show gun", &mut settings.show_gun);
    components::toggle_row(ui, "Hold to ADS", &mut settings.hold_to_ads);
    components::toggle_row(ui, "Recoil", &mut settings.recoil_enabled);
    components::toggle_row(ui, "Camera shake", &mut settings.camera_shake_enabled);
}
