use bevy_egui::egui::{self, Align, Align2, Color32, FontId, Layout, Pos2, Rect, Sense, Stroke, Vec2};

use super::stats::DashStats;
use crate::core::ui::widgets;

const NOTIF_OPEN_KEY: &str = "header_notifications_open";

/// Top horizontal header bar matching 01-MAIN_REFERENCE.jpg:
/// Active view title on the left; quick-search bar, notification icon, and
/// player profile avatar on the right.
pub fn render_header(
    ui: &mut egui::Ui,
    title: &str,
    search_query: &mut String,
    stats: &DashStats,
) {
    let mut bell_rect = Rect::NOTHING;

    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), 38.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.label(
                egui::RichText::new(title)
                    .size(22.0)
                    .strong()
                    .color(widgets::TEXT),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 10.0;

                // 1. Player Profile Avatar
                render_avatar(ui, stats);

                // 2. Notification / Patch-notes Icon Button
                bell_rect = render_notification_button(ui);

                // 3. Quick-Search Bar
                let search_w = if ui.available_width() > 380.0 { 220.0 } else { 150.0 };
                render_search_bar(ui, search_query, search_w);
            });
        },
    );

    // Optional patch-notes / notifications popover
    let open = ui
        .ctx()
        .data(|d| d.get_temp::<bool>(egui::Id::new(NOTIF_OPEN_KEY)))
        .unwrap_or(false);
    if open && bell_rect.width() > 0.0 {
        render_patch_notes_popover(ui.ctx(), bell_rect);
    }
}

fn render_search_bar(ui: &mut egui::Ui, query: &mut String, width: f32) {
    egui::Frame::new()
        .fill(widgets::SURFACE)
        .stroke(Stroke::new(1.0, widgets::BORDER))
        .corner_radius(egui::CornerRadius::same(17))
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.set_width(width);
            ui.set_height(22.0);
            ui.horizontal(|ui| {
                // Magnifying glass icon
                let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
                let p = ui.painter_at(icon_rect);
                let c = icon_rect.center() - Vec2::new(1.0, 1.0);
                let s = Stroke::new(1.4, widgets::TEXT_WEAK);
                p.circle_stroke(c, 4.5, s);
                p.line_segment([c + Vec2::new(3.2, 3.2), c + Vec2::new(6.0, 6.0)], s);

                let text_w = (ui.available_width() - if query.is_empty() { 4.0 } else { 20.0 })
                    .max(40.0);
                ui.add_sized(
                    Vec2::new(text_w, 20.0),
                    egui::TextEdit::singleline(query)
                        .frame(egui::Frame::NONE)
                        .hint_text(
                            egui::RichText::new("Search tasks...")
                                .size(12.5)
                                .color(widgets::TEXT_WEAK),
                        )
                        .font(FontId::proportional(12.5)),
                );

                if !query.is_empty() {
                    let clear = ui.add(
                        egui::Button::new(
                            egui::RichText::new("×").size(13.0).color(widgets::TEXT_WEAK),
                        )
                        .frame(false),
                    );
                    if clear.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        query.clear();
                    }
                }
            });
        });
}

fn render_notification_button(ui: &mut egui::Ui) -> Rect {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let fill = if response.hovered() {
        widgets::SURFACE_HOVER
    } else {
        widgets::SURFACE
    };
    let p = ui.painter_at(rect);
    let c = rect.center();
    p.circle_filled(c, 16.5, fill);
    p.circle_stroke(c, 16.5, Stroke::new(1.0, widgets::BORDER));

    // Bell glyph
    let bell_color = if response.hovered() {
        Color32::WHITE
    } else {
        widgets::TEXT
    };
    let s = Stroke::new(1.5, bell_color);
    p.circle_stroke(c - Vec2::new(0.0, 1.5), 4.5, s);
    p.rect_filled(
        Rect::from_center_size(c + Vec2::new(0.0, 1.5), Vec2::new(10.0, 4.0)),
        egui::CornerRadius::same(1),
        fill,
    );
    p.line_segment([c + Vec2::new(-5.5, 3.0), c + Vec2::new(5.5, 3.0)], s);
    p.line_segment([c + Vec2::new(-4.5, -1.5), c + Vec2::new(-5.0, 3.0)], s);
    p.line_segment([c + Vec2::new(4.5, -1.5), c + Vec2::new(5.0, 3.0)], s);
    p.circle_filled(c + Vec2::new(0.0, 5.2), 1.4, bell_color);

    // Subtle target-yellow indicator dot
    p.circle_filled(c + Vec2::new(5.5, -5.5), 3.0, widgets::TARGET_YELLOW);

    if response.clicked() {
        ui.ctx().data_mut(|d| {
            let key = egui::Id::new(NOTIF_OPEN_KEY);
            let open = d.get_temp::<bool>(key).unwrap_or(false);
            d.insert_temp(key, !open);
        });
    }
    response.on_hover_text("Patch Notes & Telemetry Status");
    rect
}

fn render_avatar(ui: &mut egui::Ui, stats: &DashStats) {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::hover());
    let p = ui.painter_at(rect);
    let c = rect.center();

    p.circle_filled(c, 16.5, widgets::SURFACE_HOVER);
    p.circle_stroke(c, 16.5, Stroke::new(1.5, widgets::TARGET_YELLOW.gamma_multiply(0.85)));
    p.text(
        c,
        Align2::CENTER_CENTER,
        "AR",
        FontId::monospace(12.0),
        Color32::WHITE,
    );

    let tip = format!(
        "Player Profile\nAccuracy: {:.1}%\nTotal Hits: {}\nToday: {} hits",
        stats.accuracy_pct(),
        widgets::thousands(stats.hits as i64),
        widgets::thousands(stats.hits_today as i64),
    );
    response.on_hover_text(tip);
}

fn render_patch_notes_popover(ctx: &egui::Context, anchor: Rect) {
    let pos = Pos2::new((anchor.right() - 260.0).max(16.0), anchor.bottom() + 8.0);
    egui::Area::new(egui::Id::new("patch_notes_popover"))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(widgets::SURFACE_2)
                .stroke(Stroke::new(1.0, widgets::BORDER))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(240.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Patch Notes")
                                .size(13.0)
                                .strong()
                                .color(widgets::TEXT),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("×")
                                            .size(13.0)
                                            .color(widgets::TEXT_WEAK),
                                    )
                                    .frame(false),
                                )
                                .clicked()
                            {
                                ctx.data_mut(|d| {
                                    d.insert_temp(egui::Id::new(NOTIF_OPEN_KEY), false);
                                });
                            }
                        });
                    });
                    ui.add_space(6.0);
                    for note in [
                        "• Multishot Ultimate quick-launch & 28-session history",
                        "• Tabular telemetry numerals across all charts",
                        "• Live task search & Training Room shortcut",
                    ] {
                        ui.label(
                            egui::RichText::new(note)
                                .size(11.5)
                                .color(widgets::TEXT_SUBTLE),
                        );
                    }
                });
        });
}
