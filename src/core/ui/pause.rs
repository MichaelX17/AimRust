use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Layout, Pos2, Rect, RichText, Sense,
    Stroke, StrokeKind, Vec2,
};
use bevy_egui::EguiContexts;

use crate::core::config::Settings;
use crate::core::hud::format_time;
use crate::core::round::RoundTimer;
use crate::core::state::AppState;
use crate::core::task_api::{SelectedTask, TaskRegistry};
use crate::core::ui::settings::settings_panel;
use crate::core::ui::widgets;

/// Tracks whether the pause modal is currently displaying inline settings.
#[derive(Resource, Default)]
pub struct PauseUiState {
    pub show_settings: bool,
}

/// Triggers a clean 1-frame Menu -> Countdown restart so every task's `OnEnter(Menu)` cleanup runs.
#[derive(Resource, Default)]
pub struct PendingRestart(pub bool);

/// Reset pause sub-view whenever entering Paused.
pub fn reset_pause_state(mut state: ResMut<PauseUiState>) {
    state.show_settings = false;
}

/// Complete a pending restart as soon as `OnEnter(Menu)` cleanup has executed.
pub fn handle_pending_restart(
    mut pending: ResMut<PendingRestart>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if pending.0 {
        pending.0 = false;
        next_state.set(AppState::Countdown);
    }
}

/// Esc in Playing opens the pause menu.
pub fn esc_pause(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Paused);
    }
}

/// Esc in Paused closes inline settings first, or resumes the round.
pub fn esc_resume(
    keys: Res<ButtonInput<KeyCode>>,
    mut pause_state: ResMut<PauseUiState>,
    mut pending_restart: ResMut<PendingRestart>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        if pause_state.show_settings {
            pause_state.show_settings = false;
        } else {
            next_state.set(AppState::Playing);
        }
    } else if !pause_state.show_settings && keys.just_pressed(KeyCode::KeyR) {
        pending_restart.0 = true;
        next_state.set(AppState::Menu);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn pause_ui(
    mut contexts: EguiContexts,
    windows: Query<&Window, With<PrimaryWindow>>,
    registry: Res<TaskRegistry>,
    selected: Res<SelectedTask>,
    timer: Res<RoundTimer>,
    mut settings: ResMut<Settings>,
    mut pause_state: ResMut<PauseUiState>,
    mut pending_restart: ResMut<PendingRestart>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return; };
    let Ok(window) = windows.single() else { return; };
    let size = crate::core::ui::page_size(ctx, window);

    let task_name = registry
        .0
        .iter()
        .find(|t| t.id == selected.0)
        .map(|t| t.name)
        .unwrap_or("Training");
    let remaining = format_time(timer.remaining.ceil().max(0.0) as u32);

    // Translucent dark-obsidian scrim over the frozen 3D arena
    egui::Area::new(egui::Id::new("pause_modal_overlay"))
        .fixed_pos(Pos2::ZERO)
        .show(ctx, |ui| {
            let (screen_rect, _) = ui.allocate_exact_size(size, Sense::hover());
            ui.painter().rect_filled(
                screen_rect,
                CornerRadius::ZERO,
                Color32::from_rgba_unmultiplied(13, 15, 20, 195),
            );

            if pause_state.show_settings {
                render_settings_modal(
                    ui,
                    screen_rect,
                    &mut settings,
                    &mut pause_state,
                    &mut next_state,
                );
            } else {
                render_main_pause_modal(
                    ui,
                    screen_rect,
                    task_name,
                    &remaining,
                    &mut pause_state,
                    &mut pending_restart,
                    &mut next_state,
                );
            }
        });
}

fn render_main_pause_modal(
    ui: &mut egui::Ui,
    screen_rect: Rect,
    task_name: &str,
    remaining: &str,
    pause_state: &mut PauseUiState,
    pending_restart: &mut PendingRestart,
    next_state: &mut NextState<AppState>,
) {
    let modal_w = (screen_rect.width() - 48.0).clamp(320.0, 410.0);
    let modal_h = 338.0_f32.min(screen_rect.height() - 32.0).max(296.0);
    let modal_rect = Rect::from_center_size(screen_rect.center(), Vec2::new(modal_w, modal_h));

    let mut modal_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(modal_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    modal_ui.set_clip_rect(screen_rect);

    widgets::fixed_card(&mut modal_ui, Vec2::new(modal_w, modal_h), |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;

        render_pause_header(ui, task_name, remaining);
        ui.add_space(18.0);

        let btn_w = ui.available_width();
        let btn_size = Vec2::new(btn_w, 44.0);

        if pause_action_button(ui, btn_size, PauseAction::Resume).clicked() {
            next_state.set(AppState::Playing);
        }
        ui.add_space(10.0);

        if pause_action_button(ui, btn_size, PauseAction::Restart).clicked() {
            pending_restart.0 = true;
            next_state.set(AppState::Menu);
        }
        ui.add_space(10.0);

        if pause_action_button(ui, btn_size, PauseAction::Settings).clicked() {
            pause_state.show_settings = true;
        }
        ui.add_space(10.0);

        if pause_action_button(ui, btn_size, PauseAction::Quit).clicked() {
            next_state.set(AppState::Menu);
        }
        ui.add_space(10.0);
    });
}

fn render_settings_modal(
    ui: &mut egui::Ui,
    screen_rect: Rect,
    settings: &mut Settings,
    pause_state: &mut PauseUiState,
    next_state: &mut NextState<AppState>,
) {
    let modal_w = (screen_rect.width() - 64.0).clamp(420.0, 680.0);
    let modal_h = (screen_rect.height() - 64.0).clamp(360.0, 620.0);
    let modal_rect = Rect::from_center_size(screen_rect.center(), Vec2::new(modal_w, modal_h));

    let mut modal_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(modal_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    modal_ui.set_clip_rect(screen_rect);

    let mut draft = settings.clone();

    widgets::fixed_card(&mut modal_ui, Vec2::new(modal_w, modal_h), |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;

        ui.horizontal(|ui| {
            ui.label(RichText::new("In-Game Settings").size(18.0).strong().color(widgets::TEXT));
            ui.add_space(8.0);
            widgets::chip(ui, "AUTO-SAVE", widgets::POSITIVE);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::ghost_button(ui, "Back").clicked() {
                    pause_state.show_settings = false;
                }
            });
        });
        ui.add_space(12.0);

        let footer_h = 44.0;
        let scroll_h = (ui.available_height() - footer_h - 14.0).max(120.0);
        egui::ScrollArea::vertical()
            .id_salt("pause_settings_scroll")
            .max_height(scroll_h)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                settings_panel(ui, &mut draft);
            });

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if widgets::ghost_button(ui, "Back to Pause").clicked() {
                pause_state.show_settings = false;
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if pause_action_button(ui, Vec2::new(160.0, 36.0), PauseAction::Resume).clicked() {
                    pause_state.show_settings = false;
                    next_state.set(AppState::Playing);
                }
            });
        });
    });

    if draft != *settings {
        *settings = draft;
    }
}

fn render_pause_header(ui: &mut egui::Ui, task_name: &str, remaining: &str) {
    ui.horizontal(|ui| {
        let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::hover());
        let p = ui.painter_at(icon_rect);
        let c = icon_rect.center();
        p.circle_filled(c, 17.0, widgets::SURFACE_2);
        p.circle_stroke(c, 17.0, Stroke::new(1.0, widgets::BORDER));
        p.rect_filled(Rect::from_center_size(c + Vec2::new(-4.0, 0.0), Vec2::new(3.5, 12.0)), CornerRadius::same(1), widgets::TARGET_YELLOW);
        p.rect_filled(Rect::from_center_size(c + Vec2::new(4.0, 0.0), Vec2::new(3.5, 12.0)), CornerRadius::same(1), widgets::TARGET_YELLOW);

        ui.add_space(8.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(RichText::new("PAUSED").size(18.0).strong().color(widgets::TEXT));
            ui.label(
                RichText::new(format!("{task_name}  ·  {remaining} left"))
                    .monospace()
                    .size(11.5)
                    .color(widgets::TEXT_SUBTLE),
            );
        });
    });
}

#[derive(Clone, Copy)]
enum PauseAction {
    Resume,
    Restart,
    Settings,
    Quit,
}

fn pause_action_button(ui: &mut egui::Ui, size: Vec2, action: PauseAction) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let hovered = response.hovered();
    let p = ui.painter_at(rect);

    let is_primary = matches!(action, PauseAction::Resume);
    let (fill, border, fg) = if is_primary {
        let bg = if response.is_pointer_button_down_on() {
            widgets::TARGET_YELLOW_DIM
        } else if hovered {
            Color32::from_rgb(255, 226, 70)
        } else {
            widgets::TARGET_YELLOW
        };
        (bg, Stroke::NONE, widgets::BG)
    } else if matches!(action, PauseAction::Quit) && hovered {
        (Color32::from_rgb(48, 26, 32), Stroke::new(1.0, widgets::NEGATIVE.gamma_multiply(0.7)), widgets::TEXT)
    } else if hovered {
        (widgets::SURFACE_HOVER, Stroke::new(1.0, Color32::from_rgb(76, 86, 102)), widgets::TEXT)
    } else {
        (widgets::SURFACE_2, Stroke::new(1.0, widgets::BORDER), widgets::TEXT)
    };

    p.rect_filled(rect, CornerRadius::same(10), fill);
    if border != Stroke::NONE {
        p.rect_stroke(rect, CornerRadius::same(10), border, StrokeKind::Inside);
    }

    let (label, keycap, icon_color) = match action {
        PauseAction::Resume => ("RESUME", Some("ESC"), widgets::BG),
        PauseAction::Restart => ("RESTART", Some("R"), widgets::ACCENT_CYAN),
        PauseAction::Settings => ("SETTINGS", None, widgets::TARGET_YELLOW),
        PauseAction::Quit => ("QUIT TO MENU", None, widgets::NEGATIVE),
    };

    let ic = Pos2::new(rect.left() + 24.0, rect.center().y);
    draw_action_icon(&p, ic, action, icon_color);

    p.text(
        Pos2::new(rect.left() + 44.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(14.0),
        fg,
    );

    if let Some(key) = keycap {
        let key_w = if key.len() > 1 { 34.0 } else { 22.0 };
        let key_rect = Rect::from_center_size(Pos2::new(rect.right() - 16.0 - key_w * 0.5, rect.center().y), Vec2::new(key_w, 20.0));
        let key_bg = if is_primary { fg.gamma_multiply(0.14) } else { widgets::SURFACE };
        p.rect_filled(key_rect, CornerRadius::same(4), key_bg);
        if !is_primary {
            p.rect_stroke(key_rect, CornerRadius::same(4), Stroke::new(1.0, widgets::BORDER), StrokeKind::Inside);
        }
        p.text(
            key_rect.center(),
            Align2::CENTER_CENTER,
            key,
            FontId::monospace(10.0),
            if is_primary { fg } else { widgets::TEXT_WEAK },
        );
    }

    response
}

fn draw_action_icon(p: &egui::Painter, c: Pos2, action: PauseAction, color: Color32) {
    let s = Stroke::new(1.6, color);
    let v = |x: f32, y: f32| c + Vec2::new(x, y);
    match action {
        PauseAction::Resume => {
            p.add(egui::Shape::convex_polygon(vec![v(-3.5, -5.5), v(5.5, 0.0), v(-3.5, 5.5)], color, Stroke::NONE));
        }
        PauseAction::Restart => {
            let pts: Vec<Pos2> = (0..=14)
                .map(|i| {
                    let a = -0.6 + (i as f32 / 14.0) * 4.8;
                    c + Vec2::new(a.cos(), a.sin()) * 6.0
                })
                .collect();
            p.add(egui::Shape::line(pts, s));
            p.line_segment([v(5.0, -3.5), v(5.5, -7.0)], s);
            p.line_segment([v(5.0, -3.5), v(1.5, -3.5)], s);
        }
        PauseAction::Settings => {
            for offset in [-4.0, 0.0, 4.0] {
                p.line_segment([v(-5.5, offset), v(5.5, offset)], s);
            }
            p.circle_filled(v(-2.0, -4.0), 1.6, color);
            p.circle_filled(v(2.5, 0.0), 1.6, color);
            p.circle_filled(v(-1.0, 4.0), 1.6, color);
        }
        PauseAction::Quit => {
            p.line_segment([v(-5.5, -5.5), v(-5.5, 5.5)], s);
            p.line_segment([v(-5.5, -5.5), v(1.5, -5.5)], s);
            p.line_segment([v(-5.5, 5.5), v(1.5, 5.5)], s);
            p.line_segment([v(-1.0, 0.0), v(5.5, 0.0)], s);
            p.line_segment([v(2.5, -3.0), v(5.5, 0.0)], s);
            p.line_segment([v(2.5, 3.0), v(5.5, 0.0)], s);
        }
    }
}
