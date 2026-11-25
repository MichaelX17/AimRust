//! Reusable card building blocks: header with icon, KPI block, painted icons.

use bevy_egui::egui::{self, Align, Color32, Layout, Rect, RichText, Sense, Stroke, Vec2};

use crate::core::ui::widgets;

#[derive(Clone, Copy)]
pub enum CardIcon {
    Trend,
    Target,
    Clipboard,
    Bars,
}

fn card_icon(ui: &mut egui::Ui, icon: CardIcon, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::hover());
    let p = ui.painter_at(rect);
    let c = rect.center();
    let s = Stroke::new(1.8, color);
    let v = |x: f32, y: f32| c + Vec2::new(x, y);
    match icon {
        CardIcon::Trend => {
            p.add(egui::Shape::line(
                vec![v(-9.0, 6.0), v(-3.0, 0.0), v(1.0, 3.0), v(9.0, -6.0)],
                s,
            ));
            p.line_segment([v(9.0, -6.0), v(4.0, -6.0)], s);
            p.line_segment([v(9.0, -6.0), v(9.0, -1.0)], s);
        }
        CardIcon::Target => {
            p.circle_stroke(c, 9.0, s);
            p.circle_stroke(c, 4.5, s);
            p.circle_filled(c, 1.6, color);
        }
        CardIcon::Clipboard => {
            p.rect_stroke(
                Rect::from_center_size(v(0.0, 1.0), Vec2::new(14.0, 17.0)),
                egui::CornerRadius::same(3),
                s,
                egui::StrokeKind::Inside,
            );
            p.rect_filled(
                Rect::from_center_size(v(0.0, -7.0), Vec2::new(7.0, 4.0)),
                egui::CornerRadius::same(1),
                color,
            );
            p.line_segment([v(-3.5, 0.0), v(3.5, 0.0)], Stroke::new(1.4, color));
            p.line_segment([v(-3.5, 4.0), v(3.5, 4.0)], Stroke::new(1.4, color));
        }
        CardIcon::Bars => {
            for (x, h) in [(-6.0, 7.0), (0.0, 13.0), (6.0, 10.0)] {
                p.rect_filled(
                    Rect::from_min_size(v(x - 2.0, 7.0 - h), Vec2::new(4.0, h)),
                    egui::CornerRadius::same(1),
                    color,
                );
            }
        }
    }
}

/// Card header: icon + title/subtitle on the left, `right` content on the right.
pub fn card_header(
    ui: &mut egui::Ui,
    icon: CardIcon,
    icon_color: Color32,
    title: &str,
    subtitle: Option<&str>,
    right: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        card_icon(ui, icon, icon_color);
        ui.add_space(2.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.label(RichText::new(title).size(14.0).strong().color(widgets::TEXT));
            if let Some(sub) = subtitle {
                ui.label(RichText::new(sub).size(11.5).color(widgets::TEXT_SUBTLE));
            }
        });
        ui.with_layout(Layout::right_to_left(Align::Center), right);
    });
}

/// Right-aligned headline metric with an optional ▲/▼ delta line beneath it.
pub fn kpi(ui: &mut egui::Ui, value: &str, delta: Option<(String, bool)>) {
    ui.with_layout(Layout::top_down(Align::Max), |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.label(
            RichText::new(value)
                .monospace()
                .size(20.0)
                .strong()
                .color(widgets::TEXT),
        );
        if let Some((text, positive)) = delta {
            let (arrow, color) = if positive {
                ("↑", widgets::POSITIVE)
            } else {
                ("↓", widgets::NEGATIVE)
            };
            ui.label(
                RichText::new(format!("{arrow} {text}"))
                    .monospace()
                    .size(11.0)
                    .strong()
                    .color(color),
            );
        }
    });
}

/// Small colored dot + label, used for chart legends.
pub fn legend_item(ui: &mut egui::Ui, color: Color32, label: &str) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), 3.5, color);
    ui.label(RichText::new(label).size(11.5).color(widgets::TEXT_SUBTLE));
    ui.add_space(8.0);
}

/// High-contrast primary CTA button in saturated target yellow (#FFD60A) with
/// dark charcoal typography and a crisp play glyph.
pub fn cta_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let font = egui::FontId::proportional(13.0);
    let text_color = widgets::BG;
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_color);
    let pad_x = 16.0;
    let icon_w = 18.0;
    let size = Vec2::new(galley.size().x + pad_x * 2.0 + icon_w, 34.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let fill = if response.is_pointer_button_down_on() {
        widgets::TARGET_YELLOW_DIM
    } else if response.hovered() {
        Color32::from_rgb(255, 226, 70)
    } else {
        widgets::TARGET_YELLOW
    };
    let p = ui.painter();
    p.rect_filled(rect, egui::CornerRadius::same(8), fill);
    let c = egui::pos2(rect.left() + pad_x + 3.0, rect.center().y);
    p.add(egui::Shape::convex_polygon(
        vec![c + Vec2::new(-4.0, -5.0), c + Vec2::new(4.5, 0.0), c + Vec2::new(-4.0, 5.0)],
        text_color,
        Stroke::NONE,
    ));
    p.galley(
        egui::pos2(rect.left() + pad_x + icon_w, rect.center().y - galley.size().y * 0.5),
        galley,
        text_color,
    );
    response
}
