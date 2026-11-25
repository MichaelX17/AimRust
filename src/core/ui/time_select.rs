use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Layout, Pos2, Rect, RichText, Sense,
    Stroke, StrokeKind, Vec2,
};
use bevy_egui::EguiContexts;

use crate::core::round::RoundDuration;
use crate::core::settings::UserSettings;
use crate::core::state::AppState;
use crate::core::task_api::{SelectedTask, TaskRegistry};
use crate::core::ui::menu::components;
use crate::core::ui::widgets;

const PRESETS: [(u32, &str, &str); 5] = [
    (30, "0:30", "30 sec"),
    (60, "1:00", "1 min"),
    (120, "2:00", "2 min"),
    (180, "3:00", "3 min"),
    (300, "5:00", "5 min"),
];

pub fn time_select_ui(
    mut contexts: EguiContexts,
    windows: Query<&Window, With<PrimaryWindow>>,
    registry: Res<TaskRegistry>,
    selected: Res<SelectedTask>,
    mut duration: ResMut<RoundDuration>,
    mut settings: ResMut<UserSettings>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return; };
    let Ok(window) = windows.single() else { return; };
    let size = crate::core::ui::page_size(ctx, window);

    if !PRESETS.iter().any(|(secs, _, _)| *secs == duration.0) {
        duration.0 = 60;
    }

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        next_state.set(AppState::TaskSelect);
        return;
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
        next_state.set(AppState::Countdown);
        return;
    }

    let task_name = registry
        .0
        .iter()
        .find(|t| t.id == selected.0)
        .map(|t| t.name)
        .unwrap_or("Selected Task");

    widgets::page(ctx, "time_select_modal", size, |ui| {
        let inner = widgets::page_inner(size);
        let (page_rect, _) = ui.allocate_exact_size(inner, Sense::hover());

        let modal_w = (page_rect.width() - 32.0).clamp(360.0, 500.0);
        let modal_h = 360.0_f32.min(page_rect.height() - 24.0).max(300.0);
        let modal_rect = Rect::from_center_size(page_rect.center(), Vec2::new(modal_w, modal_h));

        let mut modal_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(modal_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        modal_ui.set_clip_rect(page_rect);

        widgets::fixed_card(&mut modal_ui, Vec2::new(modal_w, modal_h), |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;

            render_header(ui, task_name);
            ui.add_space(16.0);

            render_duration_selector(ui, &mut duration.0);
            ui.add_space(16.0);

            render_hardcore_toggle(ui, &mut settings.doa_mode_enabled);

            // Responsive flexible spacer pushing footer down with generous bottom padding.
            let footer_h = 36.0;
            let bottom_padding = 10.0; // 10px inside card + 16px card border shrink = 26px clearance
            let remaining = (ui.available_height() - footer_h - bottom_padding).max(18.0);
            ui.add_space(remaining);

            render_footer(ui, settings.doa_mode_enabled, &mut next_state);
            ui.add_space(bottom_padding);
        });
    });
}

fn render_header(ui: &mut egui::Ui, task_name: &str) {
    ui.horizontal(|ui| {
        let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::hover());
        let p = ui.painter_at(icon_rect);
        let c = icon_rect.center();
        p.circle_filled(c, 15.0, widgets::SURFACE_2);
        p.circle_stroke(c, 15.0, Stroke::new(1.0, widgets::BORDER));
        p.circle_stroke(c, 8.0, Stroke::new(1.6, widgets::TARGET_YELLOW));
        p.line_segment([c, c + Vec2::new(0.0, -4.5)], Stroke::new(1.6, widgets::TARGET_YELLOW));
        p.line_segment([c, c + Vec2::new(3.5, 2.0)], Stroke::new(1.6, widgets::TARGET_YELLOW));

        ui.add_space(6.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.label(RichText::new("Round Setup").size(18.0).strong().color(widgets::TEXT));
            ui.label(RichText::new(task_name).size(12.0).color(widgets::TEXT_SUBTLE));
        });
    });
}

fn render_duration_selector(ui: &mut egui::Ui, selected_secs: &mut u32) {
    ui.horizontal(|ui| {
        widgets::small_label(ui, "Time per round");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let formatted = format!("{:02}:{:02}", *selected_secs / 60, *selected_secs % 60);
            ui.label(
                RichText::new(formatted)
                    .monospace()
                    .size(13.0)
                    .strong()
                    .color(widgets::TARGET_YELLOW),
            );
        });
    });
    ui.add_space(6.0);

    let gap = 8.0;
    let total_w = ui.available_width();
    let tile_w = ((total_w - gap * (PRESETS.len() - 1) as f32) / PRESETS.len() as f32).max(48.0);
    let tile_h = 58.0;

    let (row_rect, _) = ui.allocate_exact_size(Vec2::new(total_w, tile_h), Sense::hover());
    for (idx, (secs, time_str, caption)) in PRESETS.iter().enumerate() {
        let rect = Rect::from_min_size(
            Pos2::new(row_rect.left() + idx as f32 * (tile_w + gap), row_rect.top()),
            Vec2::new(tile_w, tile_h),
        );
        let response = ui
            .interact(rect, ui.id().with(("preset", *secs)), Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        let active = *selected_secs == *secs;
        let (fill, stroke, time_color) = if active {
            (widgets::SURFACE_HOVER, Stroke::new(1.5, widgets::TARGET_YELLOW), Color32::WHITE)
        } else if response.hovered() {
            (widgets::SURFACE_HOVER, Stroke::new(1.0, Color32::from_rgb(72, 80, 96)), widgets::TEXT)
        } else {
            (widgets::SURFACE_2, Stroke::new(1.0, widgets::BORDER), widgets::TEXT)
        };

        let p = ui.painter_at(rect);
        p.rect_filled(rect, CornerRadius::same(10), fill);
        p.rect_stroke(rect, CornerRadius::same(10), stroke, StrokeKind::Inside);

        if active {
            p.rect_filled(
                Rect::from_min_size(Pos2::new(rect.center().x - 14.0, rect.top() + 2.0), Vec2::new(28.0, 2.5)),
                CornerRadius::same(1),
                widgets::TARGET_YELLOW,
            );
        }
        p.text(Pos2::new(rect.center().x, rect.top() + 23.0), Align2::CENTER_CENTER, *time_str, FontId::monospace(15.5), time_color);
        p.text(
            Pos2::new(rect.center().x, rect.bottom() - 14.0),
            Align2::CENTER_CENTER,
            *caption,
            FontId::proportional(10.5),
            if active { widgets::TARGET_YELLOW } else { widgets::TEXT_WEAK },
        );

        if response.clicked() {
            *selected_secs = *secs;
        }
    }
}

fn render_hardcore_toggle(ui: &mut egui::Ui, enabled: &mut bool) {
    let total_w = ui.available_width();
    let row_h = 64.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(total_w, row_h), Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    if response.clicked() {
        *enabled = !*enabled;
    }

    let p = ui.painter_at(rect);
    let fill = if *enabled {
        Color32::from_rgb(44, 24, 30)
    } else if response.hovered() {
        widgets::SURFACE_HOVER
    } else {
        widgets::SURFACE_2
    };
    let border = if *enabled { widgets::NEGATIVE.gamma_multiply(0.8) } else { widgets::BORDER };

    p.rect_filled(rect, CornerRadius::same(10), fill);
    p.rect_stroke(rect, CornerRadius::same(10), Stroke::new(1.0, border), StrokeKind::Inside);

    let bar_color = if *enabled { widgets::NEGATIVE } else { widgets::TEXT_WEAK };
    p.rect_filled(
        Rect::from_min_size(rect.min + Vec2::new(10.0, 14.0), Vec2::new(3.0, row_h - 28.0)),
        CornerRadius::same(2),
        bar_color,
    );

    let text_x = rect.left() + 22.0;
    let title_galley = ui.painter().layout_no_wrap("Hardcore Mode".to_string(), FontId::proportional(14.0), Color32::WHITE);
    let title_w = title_galley.size().x;
    p.galley(Pos2::new(text_x, rect.top() + 22.0 - title_galley.size().y * 0.5), title_galley, Color32::WHITE);

    let tag_text = if *enabled { "DEAD OR ALIVE" } else { "OFF" };
    let tag_color = if *enabled { widgets::NEGATIVE } else { widgets::TEXT_WEAK };
    let tag_rect = Rect::from_min_size(
        Pos2::new(text_x + title_w + 10.0, rect.top() + 12.0),
        Vec2::new(if *enabled { 92.0 } else { 36.0 }, 18.0),
    );
    p.rect_filled(tag_rect, CornerRadius::same(4), tag_color.gamma_multiply(0.18));
    p.rect_stroke(tag_rect, CornerRadius::same(4), Stroke::new(1.0, tag_color.gamma_multiply(0.5)), StrokeKind::Inside);
    p.text(tag_rect.center(), Align2::CENTER_CENTER, tag_text, FontId::monospace(9.5), tag_color);

    p.text(
        Pos2::new(text_x, rect.bottom() - 17.0),
        Align2::LEFT_CENTER,
        "One missed shot immediately ends the round",
        FontId::proportional(11.5),
        widgets::TEXT_SUBTLE,
    );

    let switch_w = 46.0;
    let switch_h = 24.0;
    let switch_rect = Rect::from_center_size(
        Pos2::new(rect.right() - 16.0 - switch_w * 0.5, rect.center().y),
        Vec2::new(switch_w, switch_h),
    );
    let track_fill = if *enabled { widgets::NEGATIVE } else { widgets::SURFACE_HOVER };
    p.rect_filled(switch_rect, CornerRadius::same(12), track_fill);
    p.rect_stroke(
        switch_rect,
        CornerRadius::same(12),
        Stroke::new(1.0, if *enabled { widgets::NEGATIVE } else { widgets::BORDER }),
        StrokeKind::Inside,
    );
    let knob_x = if *enabled { switch_rect.right() - 12.0 } else { switch_rect.left() + 12.0 };
    p.circle_filled(Pos2::new(knob_x, switch_rect.center().y), 8.5, Color32::WHITE);
}

fn render_footer(ui: &mut egui::Ui, hardcore: bool, next_state: &mut NextState<AppState>) {
    ui.horizontal(|ui| {
        if widgets::ghost_button(ui, "Back").clicked() {
            next_state.set(AppState::TaskSelect);
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let cta = if hardcore { "START HARDCORE" } else { "START ROUND" };
            if components::cta_button(ui, cta).clicked() {
                next_state.set(AppState::Countdown);
            }
        });
    });
}
