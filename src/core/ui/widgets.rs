//! Shared visual language for the dashboard-style UI.

use bevy_egui::egui;
use egui::{Color32, CornerRadius, Margin, RichText, Stroke};

pub const PAGE_MARGIN: f32 = 18.0;
pub const BG: Color32 = Color32::from_rgb(10, 10, 16);
pub const SURFACE: Color32 = Color32::from_rgb(17, 17, 26);
pub const SURFACE_2: Color32 = Color32::from_rgb(25, 25, 38);
pub const BORDER: Color32 = Color32::from_rgb(38, 38, 56);
pub const TEXT: Color32 = Color32::from_rgb(232, 232, 244);
pub const TEXT_WEAK: Color32 = Color32::from_rgb(142, 142, 164);
pub const POSITIVE: Color32 = Color32::from_rgb(52, 211, 153);
pub const NEGATIVE: Color32 = Color32::from_rgb(248, 113, 113);

/// Full-page dark background sized by the real window size.
pub fn page_inner(size: egui::Vec2) -> egui::Vec2 {
    egui::vec2(
        (size.x - PAGE_MARGIN * 2.0).max(200.0),
        (size.y - PAGE_MARGIN * 2.0).max(200.0),
    )
}

pub fn page(
    ctx: &egui::Context,
    id: &'static str,
    size: egui::Vec2,
    add: impl FnOnce(&mut egui::Ui),
) {
    egui::Area::new(egui::Id::new(id))
        .fixed_pos(egui::Pos2::ZERO)
        .show(ctx, |ui| {
            egui::Frame::new().fill(BG).show(ui, |ui| {
                let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                let inner_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2(PAGE_MARGIN, PAGE_MARGIN),
                    page_inner(size),
                );
                let mut child = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(inner_rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );
                child.set_clip_rect(rect);
                add(&mut child);
            });
        });
}

/// Card painted at an exact size; content is clipped to it.
pub fn fixed_card(ui: &mut egui::Ui, size: egui::Vec2, add: impl FnOnce(&mut egui::Ui)) {
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, egui::CornerRadius::same(14), SURFACE);
    painter.rect_stroke(
        rect,
        egui::CornerRadius::same(14),
        egui::Stroke::new(1.0, BORDER),
        egui::StrokeKind::Inside,
    );

    let mut content = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink(16.0))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    // Keep the card's own clipping but never paint outside the parent's
    // viewport (e.g. when a column of cards is scrolled).
    content.set_clip_rect(ui.clip_rect().intersect(rect));
    add(&mut content);
}

/// Rounded dark card.
pub fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(16))
}

/// Smaller inset tile, used for stats inside cards.
pub fn tile() -> egui::Frame {
    egui::Frame::new()
        .fill(SURFACE_2)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(10))
}

pub fn small_label(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(
        RichText::new(text.into().to_uppercase())
            .size(11.0)
            .strong()
            .color(TEXT_WEAK),
    );
}

pub fn muted(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(RichText::new(text.into()).size(13.0).color(TEXT_WEAK));
}

pub fn big_value(ui: &mut egui::Ui, text: impl Into<String>, size: f32) {
    ui.label(
        RichText::new(text.into())
            .size(size)
            .strong()
            .color(TEXT),
    );
}

pub fn chip(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.16))
        .stroke(Stroke::new(1.0, color.gamma_multiply(0.45)))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.0).strong().color(color));
        });
}

fn text_on(fill: Color32) -> Color32 {
    let [r, g, b, _] = fill.to_array();
    if (r as u16 + g as u16 + b as u16) / 3 > 170 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

pub fn primary_button(ui: &mut egui::Ui, text: &str, accent: Color32) -> egui::Response {
    ui.add(
        egui::Button::new(
            RichText::new(text)
                .size(14.0)
                .strong()
                .color(text_on(accent)),
        )
        .fill(accent)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(10))
        .min_size(egui::vec2(0.0, 38.0)),
    )
}

pub fn ghost_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).size(13.0).color(TEXT))
            .fill(SURFACE_2)
            .stroke(Stroke::new(1.0, BORDER))
            .corner_radius(CornerRadius::same(10))
            .min_size(egui::vec2(0.0, 34.0)),
    )
}

/// Minimal bar chart drawn with the painter.
pub fn bar_chart(
    ui: &mut egui::Ui,
    values: &[f32],
    accent: Color32,
    height: f32,
) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    if values.is_empty() {
        return;
    }
    let painter = ui.painter_at(rect);
    let max = values.iter().copied().fold(0.0f32, f32::max).max(1.0);
    let gap = 4.0;
    let count = values.len() as f32;
    let bar_width = ((rect.width() - gap * (count - 1.0)) / count).clamp(2.0, 26.0);
    let total_width = count * bar_width + gap * (count - 1.0);
    let start_x = rect.left() + ((rect.width() - total_width) * 0.5).max(0.0);

    for (index, value) in values.iter().enumerate() {
        let ratio = (value / max).clamp(0.03, 1.0);
        let bar_height = (rect.height() - 6.0) * ratio;
        let x = start_x + index as f32 * (bar_width + gap);
        let bar = egui::Rect::from_min_max(
            egui::pos2(x, rect.bottom() - bar_height),
            egui::pos2(x + bar_width, rect.bottom()),
        );
        let last = index + 1 == values.len();
        let color = if last {
            accent
        } else {
            accent.gamma_multiply(0.55)
        };
        painter.rect_filled(bar, CornerRadius::same(3), color);
    }
}

/// Formats an integer with thousands separators.
pub fn thousands(value: i64) -> String {
    let negative = value < 0;
    let digits = value.unsigned_abs().to_string();
    let mut out = String::new();
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if negative {
        out.insert(0, '-');
    }
    out
}
