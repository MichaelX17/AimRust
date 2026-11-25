pub mod aim;
pub mod audio;
pub mod colors;
pub mod components;
pub mod crosshair;
pub mod icons;
pub mod video_gameplay;

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::EguiContexts;
use bevy_egui::egui::{self, Align, Layout, Pos2, Rect, Vec2};

use crate::core::config::Settings;
use crate::core::state::AppState;
use crate::core::ui::UiAccent;
use crate::core::ui::menu::sidebar;
use crate::core::ui::widgets;

const SIDEBAR_WIDTH: f32 = 200.0;
const SIDEBAR_NARROW: f32 = 168.0;
const SIDEBAR_GAP: f32 = 16.0;
const CARD_GAP: f32 = 14.0;
/// Ergonomic max width for the Settings grid so cards never stretch excessively on ultrawide monitors.
const MAX_SETTINGS_WIDTH: f32 = 1080.0;

pub fn settings_screen(
    mut contexts: EguiContexts,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut settings: ResMut<Settings>,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
    accent: Res<UiAccent>,
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
        let inner = widgets::page_inner(size);
        let sidebar_width = if inner.x < 1050.0 {
            SIDEBAR_NARROW
        } else {
            SIDEBAR_WIDTH
        };

        let (page_rect, _) = ui.allocate_exact_size(inner, egui::Sense::hover());
        let sidebar_rect =
            Rect::from_min_size(page_rect.min, Vec2::new(sidebar_width, page_rect.height()));
        let content_width = (page_rect.width() - sidebar_width - SIDEBAR_GAP).max(0.0);
        let content_rect = Rect::from_min_size(
            Pos2::new(sidebar_rect.right() + SIDEBAR_GAP, page_rect.top()),
            Vec2::new(content_width, page_rect.height()),
        );

        // 1. Left Navigation Sidebar
        let mut sidebar_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(sidebar_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        sidebar_ui.set_clip_rect(page_rect);
        sidebar::render_sidebar(
            &mut sidebar_ui,
            sidebar::NavTab::Settings,
            accent.0,
            page_rect.height(),
            sidebar_width,
            &mut next_state,
            &mut exit,
        );

        // 2. Main Settings Area
        let mut content_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        content_ui.set_clip_rect(page_rect);

        egui::ScrollArea::vertical()
            .max_width(content_width)
            .max_height(page_rect.height())
            .auto_shrink([false, false])
            .show(&mut content_ui, |ui| {
                // Constrain content to MAX_SETTINGS_WIDTH and center horizontally on ultrawide displays
                let avail_w = ui.available_width();
                let container_w = avail_w.min(MAX_SETTINGS_WIDTH).floor().max(240.0);
                let side_pad = ((avail_w - container_w) * 0.5).max(0.0);

                ui.horizontal(|ui| {
                    if side_pad > 0.0 {
                        ui.add_space(side_pad);
                    }
                    ui.vertical(|ui| {
                        ui.set_width(container_w);

                        render_settings_header(ui);
                        ui.add_space(14.0);

                        render_settings_grid(ui, container_w, &mut draft);
                        ui.add_space(16.0);

                        ui.vertical_centered(|ui| {
                            if components::reset_defaults_button(ui).clicked() {
                                draft = Settings::default();
                            }
                        });
                        ui.add_space(14.0);
                    });
                });
            });
    });

    if draft != *settings {
        *settings = draft;
    }
}

fn render_settings_header(ui: &mut egui::Ui) {
    if ui.available_width() < 420.0 {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Settings")
                    .size(22.0)
                    .strong()
                    .color(widgets::TEXT),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                components::render_autosave_pill(ui);
            });
        });
        ui.label(
            egui::RichText::new("Customize AimRust to fit your playstyle")
                .size(12.0)
                .color(widgets::TEXT_SUBTLE),
        );
    } else {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("Settings")
                        .size(24.0)
                        .strong()
                        .color(widgets::TEXT),
                );
                ui.label(
                    egui::RichText::new("Customize AimRust to fit your playstyle")
                        .size(12.5)
                        .color(widgets::TEXT_SUBTLE),
                );
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                components::render_autosave_pill(ui);
            });
        });
    }
}

/// Asymmetric Bento layout modeled after 01-MAIN_REFERENCE.jpg:
/// - Wide (>= 760px):
///   - Row 0: Crosshair hero card (58%) | Aim + Audio stacked flush (42%)
///   - Row 1: Gameplay (50%) | Video (50%)
///   - Row 2: Environment (50%) | Target & UI (50%)
/// - Narrow (< 760px): clean single-column vertical stack with exact content-fitted heights.
fn render_settings_grid(ui: &mut egui::Ui, width: f32, draft: &mut Settings) {
    const PAIR_H: f32 = 138.0; // 2-row cards (Aim, Audio)
    const HERO_H: f32 = PAIR_H * 2.0 + CARD_GAP; // 290.0 — flush with Aim + gap + Audio
    const ROW1_H: f32 = 212.0; // 4-row Gameplay + 3-row Video
    const ROW2_H: f32 = 176.0; // 3-row Environment + 2-row Target & UI

    if width >= 760.0 {
        let half_w = ((width - CARD_GAP) * 0.5).floor();
        let half_w_right = (width - CARD_GAP - half_w).floor();

        // Row 0: Crosshair (50%) | Aim + Audio stacked flush (50%) — aligned vertical gutter
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = CARD_GAP;

            crosshair::render_crosshair_card(ui, half_w, HERO_H, draft);

            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = CARD_GAP;
                aim::render_aim_card(ui, half_w_right, PAIR_H, draft);
                audio::render_audio_card(ui, half_w_right, PAIR_H, draft);
            });
        });

        ui.add_space(CARD_GAP);

        // Row 1: Gameplay (50%) | Video (50%)
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = CARD_GAP;
            video_gameplay::render_gameplay_card(ui, half_w, ROW1_H, draft);
            video_gameplay::render_video_card(ui, half_w_right, ROW1_H, draft);
        });

        ui.add_space(CARD_GAP);

        // Row 2: Environment (50%) | Target & UI (50%)
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = CARD_GAP;
            colors::render_environment_card(ui, half_w, ROW2_H, draft);
            colors::render_target_ui_card(ui, half_w_right, ROW2_H, draft);
        });
    } else {
        // Narrow single-column stack
        let col_w = width.floor().max(240.0);
        let crosshair_h = if col_w >= 416.0 { HERO_H } else { 370.0 };

        aim::render_aim_card(ui, col_w, PAIR_H, draft);
        ui.add_space(CARD_GAP);
        audio::render_audio_card(ui, col_w, PAIR_H, draft);
        ui.add_space(CARD_GAP);
        crosshair::render_crosshair_card(ui, col_w, crosshair_h, draft);
        ui.add_space(CARD_GAP);
        video_gameplay::render_gameplay_card(ui, col_w, ROW1_H, draft);
        ui.add_space(CARD_GAP);
        video_gameplay::render_video_card(ui, col_w, ROW2_H, draft);
        ui.add_space(CARD_GAP);
        colors::render_environment_card(ui, col_w, ROW2_H, draft);
        ui.add_space(CARD_GAP);
        colors::render_target_ui_card(ui, col_w, PAIR_H, draft);
    }
}

/// Stacked settings panel used by the Sandbox debug window.
pub fn settings_panel(ui: &mut egui::Ui, settings: &mut Settings) {
    section(ui, "Aim", |ui| aim::aim_controls(ui, settings));
    ui.add_space(12.0);
    section(ui, "Audio", |ui| audio::audio_controls(ui, settings));
    ui.add_space(12.0);
    section(ui, "Crosshair", |ui| crosshair::crosshair_controls(ui, settings));
    ui.add_space(12.0);
    section(ui, "Gameplay", |ui| video_gameplay::gameplay_controls(ui, settings));
    ui.add_space(12.0);
    section(ui, "Video", |ui| video_gameplay::video_controls(ui, settings));
    ui.add_space(12.0);
    section(ui, "Environment", |ui| colors::environment_controls(ui, settings));
    ui.add_space(12.0);
    section(ui, "Target & UI", |ui| colors::target_ui_controls(ui, settings));
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        if components::reset_defaults_button(ui).clicked() {
            *settings = Settings::default();
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            widgets::muted(ui, "Changes are saved automatically");
        });
    });
}

fn section(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    widgets::card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = components::ROW_GAP;
        widgets::small_label(ui, title);
        ui.add_space(4.0);
        add_contents(ui);
    });
}
