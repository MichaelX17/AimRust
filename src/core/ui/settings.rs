use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::{EguiContexts, egui};

use crate::core::config::{CrosshairStyle, Settings, WindowMode};
use crate::core::state::AppState;
use crate::core::ui::widgets;

pub fn settings_screen(
    mut contexts: EguiContexts,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut settings: ResMut<Settings>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let size = crate::core::ui::page_size(ctx, window);

    // Edit a draft so mere UI rendering does not mark `Settings` as changed
    // (which would reset the autosave debounce every frame).
    let mut draft = settings.clone();

    widgets::page(ctx, "settings_screen", size, |ui| {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if widgets::ghost_button(ui, "<  BACK").clicked() {
                next_state.set(AppState::Menu);
            }
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Settings")
                    .size(26.0)
                    .strong()
                    .color(widgets::TEXT),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                widgets::chip(ui, "AUTO-SAVE ON", widgets::POSITIVE);
            });
        });
        ui.add_space(12.0);

        let available = ui.available_size();
        let gap = 16.0;
        let reset_height = 52.0;
        let grid_area_height = (available.y - reset_height).max(220.0);
        let card_width = ((available.x - gap) * 0.5).clamp(280.0, 520.0);
        let card_height = ((grid_area_height - gap * 2.0 - 18.0) / 3.0).clamp(160.0, 220.0);
        let grid_width = card_width * 2.0 + gap;
        let grid_height = card_height * 3.0 + gap * 2.0;
        let pad_x = ((available.x - grid_width) * 0.5).max(0.0);
        let pad_y = ((grid_area_height - grid_height) * 0.5).max(0.0);

        egui::ScrollArea::vertical()
            .max_height(grid_area_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(pad_x);
                    ui.vertical(|ui| {
                        ui.add_space(pad_y);

                        ui.horizontal(|ui| {
                            card_cell(ui, card_width, card_height, "Aim", |ui| {
                                aim_controls(ui, &mut draft);
                            });
                            ui.add_space(gap);
                            card_cell(ui, card_width, card_height, "Crosshair", |ui| {
                                crosshair_controls(ui, &mut draft);
                            });
                        });
                        ui.add_space(gap);

                        ui.horizontal(|ui| {
                            card_cell(ui, card_width, card_height, "Environment", |ui| {
                                environment_controls(ui, &mut draft);
                            });
                            ui.add_space(gap);
                            card_cell(ui, card_width, card_height, "Target & UI", |ui| {
                                target_ui_controls(ui, &mut draft);
                            });
                        });
                        ui.add_space(gap);

                        ui.horizontal(|ui| {
                            card_cell(ui, card_width, card_height, "Video", |ui| {
                                video_controls(ui, &mut draft);
                            });
                            ui.add_space(gap);
                            card_cell(ui, card_width, card_height, "Gameplay", |ui| {
                                gameplay_controls(ui, &mut draft, false);
                            });
                        });
                    });
                });
            });

        ui.add_space(8.0);
        ui.vertical_centered(|ui| {
            if widgets::ghost_button(ui, "RESET TO DEFAULTS").clicked() {
                draft = Settings::default();
            }
        });
    });

    if draft != *settings {
        *settings = draft;
    }
}

/// Equal-sized settings card in the 2x3 grid.
fn card_cell(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(width, height),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    painter.rect_filled(rect, egui::CornerRadius::same(14), widgets::SURFACE);
    painter.rect_stroke(
        rect,
        egui::CornerRadius::same(14),
        egui::Stroke::new(1.0, widgets::BORDER),
        egui::StrokeKind::Inside,
    );

    let mut content = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink(16.0))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    content.set_clip_rect(rect);
    widgets::small_label(&mut content, title);
    content.add_space(8.0);
    add_contents(&mut content);
}

/// Stacked settings panel used by the Sandbox debug window.
pub fn settings_panel(ui: &mut egui::Ui, settings: &mut Settings) {
    section(ui, "Aim", |ui| aim_controls(ui, settings));
    ui.add_space(12.0);
    section(ui, "Crosshair", |ui| crosshair_controls(ui, settings));
    ui.add_space(12.0);
    section(ui, "Environment", |ui| environment_controls(ui, settings));
    ui.add_space(12.0);
    section(ui, "Target & UI", |ui| target_ui_controls(ui, settings));
    ui.add_space(12.0);
    section(ui, "Video", |ui| video_controls(ui, settings));
    ui.add_space(12.0);
    section(ui, "Gameplay", |ui| gameplay_controls(ui, settings, false));
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        if widgets::ghost_button(ui, "RESET TO DEFAULTS").clicked() {
            *settings = Settings::default();
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            widgets::muted(ui, "Changes are saved automatically");
        });
    });
}

fn section(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    widgets::card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        widgets::small_label(ui, title);
        ui.add_space(8.0);
        add_contents(ui);
    });
}

fn aim_controls(ui: &mut egui::Ui, settings: &mut Settings) {
    egui::Grid::new("aim_grid")
        .num_columns(2)
        .spacing([12.0, 10.0])
        .show(ui, |ui| {
            row_label(ui, "Sensitivity");
            ui.add(egui::Slider::new(&mut settings.sens, 0.1..=10.0));
            ui.end_row();

            row_label(ui, "FOV (horizontal)");
            ui.add(egui::Slider::new(&mut settings.fov_h, 60.0..=130.0));
            ui.end_row();
        });
}

fn crosshair_controls(ui: &mut egui::Ui, settings: &mut Settings) {
    egui::Grid::new("crosshair_grid")
        .num_columns(4)
        .spacing([10.0, 10.0])
        .show(ui, |ui| {
            row_label(ui, "Size");
            ui.add(egui::Slider::new(&mut settings.crosshair.size, 1.0..=40.0));
            row_label(ui, "Style");
            egui::ComboBox::from_id_salt("crosshair_style")
                .selected_text(format!("{:?}", settings.crosshair.style))
                .show_ui(ui, |ui| {
                    for style in [
                        CrosshairStyle::Dot,
                        CrosshairStyle::Cross,
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
            ui.end_row();

            row_label(ui, "Thickness");
            ui.add(egui::Slider::new(
                &mut settings.crosshair.thickness,
                1.0..=10.0,
            ));
            row_label(ui, "Color");
            ui.color_edit_button_srgb(&mut settings.crosshair.color);
            ui.end_row();

            row_label(ui, "Gap");
            ui.add(egui::Slider::new(&mut settings.crosshair.gap, 0.0..=30.0));
            row_label(ui, "Outline");
            ui.checkbox(&mut settings.crosshair.outline, "");
            ui.end_row();
        });
}

fn environment_controls(ui: &mut egui::Ui, settings: &mut Settings) {
    egui::Grid::new("environment_grid")
        .num_columns(2)
        .spacing([12.0, 10.0])
        .show(ui, |ui| {
            row_label(ui, "Floor");
            ui.color_edit_button_srgb(&mut settings.colors.floor);
            ui.end_row();

            row_label(ui, "Ceiling");
            ui.color_edit_button_srgb(&mut settings.colors.ceiling);
            ui.end_row();

            row_label(ui, "Walls");
            ui.color_edit_button_srgb(&mut settings.colors.walls);
            ui.end_row();
        });
}

fn target_ui_controls(ui: &mut egui::Ui, settings: &mut Settings) {
    egui::Grid::new("target_ui_grid")
        .num_columns(2)
        .spacing([12.0, 10.0])
        .show(ui, |ui| {
            row_label(ui, "Target");
            ui.color_edit_button_srgb(&mut settings.colors.target);
            ui.end_row();

            row_label(ui, "UI background / accent");
            ui.color_edit_button_srgb(&mut settings.colors.background_accent);
            ui.end_row();
        });
}

fn video_controls(ui: &mut egui::Ui, settings: &mut Settings) {
    ui.horizontal(|ui| {
        row_label(ui, "Vsync");
        ui.checkbox(&mut settings.video.vsync, "");
        ui.add_space(16.0);
        row_label(ui, "FPS cap");
        ui.add(egui::Slider::new(&mut settings.video.fps_cap, 0..=360));
    });

    ui.add_space(10.0);
    ui.horizontal(|ui| {
        row_label(ui, "Window mode");
        egui::ComboBox::from_id_salt("window_mode")
            .selected_text(format!("{:?}", settings.video.window_mode))
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
}

fn gameplay_controls(ui: &mut egui::Ui, settings: &mut Settings, with_reset: bool) {
    egui::Grid::new("misc_grid")
        .num_columns(2)
        .spacing([12.0, 10.0])
        .show(ui, |ui| {
            row_label(ui, "Show gun");
            ui.checkbox(&mut settings.show_gun, "");
            ui.end_row();

            row_label(ui, "Hold to ADS");
            ui.checkbox(&mut settings.hold_to_ads, "");
            ui.end_row();
        });

    if with_reset {
        ui.add_space(10.0);
        if widgets::ghost_button(ui, "RESET TO DEFAULTS").clicked() {
            *settings = Settings::default();
        }
    }
}

fn row_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .size(13.0)
            .color(widgets::TEXT_WEAK),
    );
}
