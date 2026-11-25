use bevy_egui::egui::{self, Align, Layout};

use crate::core::config::{CrosshairStyle, Settings};
use crate::core::ui::crosshair_preview::render_crosshair_preview_box_sized;
use crate::core::ui::widgets;
use super::components::{self, ROW_GAP, ROW_HEIGHT, SettingsCategory};

pub fn render_crosshair_card(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    settings: &mut Settings,
) {
    components::settings_card(
        ui,
        width,
        height,
        SettingsCategory::Crosshair,
        "CROSSHAIR",
        |ui| {
            crosshair_controls(ui, settings);
        },
    );
}

pub fn crosshair_controls(ui: &mut egui::Ui, settings: &mut Settings) {
    let avail = ui.available_width();

    if avail >= 360.0 {
        // Hero Crosshair layout: left column with all 6 aligned controls, right column with Live Preview
        let preview_col_w = 122.0;
        let col_gap = 16.0;
        let controls_w = (avail - preview_col_w - col_gap).max(200.0);

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = col_gap;

            ui.allocate_ui_with_layout(
                egui::vec2(controls_w, 6.0 * ROW_HEIGHT + 5.0 * ROW_GAP),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.set_min_width(controls_w);
                    ui.set_max_width(controls_w);
                    ui.spacing_mut().item_spacing.y = ROW_GAP;
                    crosshair_control_rows(ui, settings);
                },
            );

            ui.allocate_ui_with_layout(
                egui::vec2(preview_col_w, 6.0 * ROW_HEIGHT + 5.0 * ROW_GAP),
                Layout::top_down(Align::Center),
                |ui| {
                    ui.add_space(32.0);
                    render_crosshair_preview_box_sized(ui, &settings.crosshair, 106.0);
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("LIVE PREVIEW")
                            .size(10.5)
                            .strong()
                            .color(widgets::TEXT_SUBTLE),
                    );
                },
            );
        });
    } else {
        // Narrow stacked layout: centered preview on top, then all 6 labeled control rows
        ui.vertical_centered(|ui| {
            render_crosshair_preview_box_sized(ui, &settings.crosshair, 84.0);
        });
        ui.add_space(4.0);
        crosshair_control_rows(ui, settings);
    }
}

fn crosshair_control_rows(ui: &mut egui::Ui, settings: &mut Settings) {
    components::slider_row(ui, "Size", &mut settings.crosshair.size, 1.0..=40.0, 1, None);
    components::slider_row(
        ui,
        "Thickness",
        &mut settings.crosshair.thickness,
        1.0..=10.0,
        1,
        None,
    );
    components::slider_row(ui, "Gap", &mut settings.crosshair.gap, 0.0..=30.0, 1, None);

    // Style dropdown row
    let row_w = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(row_w, ROW_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.add(
                egui::Label::new(
                    egui::RichText::new("Style")
                        .size(13.0)
                        .color(widgets::TEXT_WEAK),
                )
                .truncate(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                egui::ComboBox::from_id_salt("crosshair_style")
                    .selected_text(format!("{:?}", settings.crosshair.style))
                    .width(108.0)
                    .show_ui(ui, |ui| {
                        for style in [
                            CrosshairStyle::Cross,
                            CrosshairStyle::Dot,
                            CrosshairStyle::Circle,
                            CrosshairStyle::T,
                        ] {
                            ui.selectable_value(
                                &mut settings.crosshair.style,
                                style,
                                format!("{style:?}"),
                            );
                        }
                    });
            });
        },
    );

    // Color swatch row
    ui.allocate_ui_with_layout(
        egui::vec2(row_w, ROW_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.add(
                egui::Label::new(
                    egui::RichText::new("Color")
                        .size(13.0)
                        .color(widgets::TEXT_WEAK),
                )
                .truncate(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                components::color_swatch(ui, &mut settings.crosshair.color, 56.0, 22.0);
            });
        },
    );

    // Outline toggle row
    components::toggle_row(ui, "Outline", &mut settings.crosshair.outline);
}
