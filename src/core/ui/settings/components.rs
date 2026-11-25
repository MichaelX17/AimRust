use bevy_egui::egui::{self, Align, CornerRadius, FontId, Layout, Pos2, Sense, Stroke, StrokeKind, Vec2};

use crate::core::ui::widgets;
pub use super::icons::{SettingsCategory, draw_category_icon};

pub const ROW_HEIGHT: f32 = 26.0;
pub const ROW_GAP: f32 = 8.0;

/// Renders a unified Bento card for the settings grid with deterministic vertical rhythm.
pub fn settings_card<R>(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    category: SettingsCategory,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), Sense::hover());
    let painter = ui.painter();

    // Dark elevated card surface matching 01-MAIN_REFERENCE.jpg and dashboard
    painter.rect_filled(rect, CornerRadius::same(14), widgets::SURFACE);
    painter.rect_stroke(
        rect,
        CornerRadius::same(14),
        Stroke::new(1.0, widgets::BORDER),
        StrokeKind::Inside,
    );

    let mut inner = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(egui::vec2(18.0, 14.0)))
            .layout(Layout::top_down(Align::Min)),
    );
    inner.set_clip_rect(ui.clip_rect().intersect(rect));
    inner.spacing_mut().item_spacing.y = ROW_GAP;

    // Card Header: Icon + Title
    inner.allocate_ui_with_layout(
        egui::vec2(inner.available_width(), 20.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), Sense::hover());
            draw_category_icon(ui.painter(), icon_rect, category);
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(title)
                    .size(13.0)
                    .strong()
                    .color(widgets::TEXT),
            );
        },
    );

    // Subtle 1px hairline divider (egui adds ROW_GAP above and below automatically)
    let (div_rect, _) = inner.allocate_exact_size(egui::vec2(inner.available_width(), 1.0), Sense::hover());
    inner.painter().rect_filled(
        div_rect,
        CornerRadius::ZERO,
        widgets::BORDER.gamma_multiply(0.65),
    );

    add_contents(&mut inner)
}

/// Strictly left-aligned fixed-width label cell (avoids both `add_sized` centering and shrink-to-fit drift).
pub fn left_label(ui: &mut egui::Ui, text: &str, width: f32) {
    ui.allocate_ui_with_layout(
        egui::vec2(width, ROW_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.set_min_width(width);
            ui.set_max_width(width);
            ui.add(
                egui::Label::new(
                    egui::RichText::new(text)
                        .size(13.0)
                        .color(widgets::TEXT_WEAK),
                )
                .truncate(),
            );
        },
    );
}

/// Sleek pill toggle switch locked to AimRust's accent & surface tokens.
pub fn toggle_switch(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = egui::vec2(38.0, 20.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());
    response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    let enabled = ui.is_enabled();
    let hovered = response.hovered() && enabled;
    let painter = ui.painter();

    let bg = if *on {
        if enabled {
            widgets::ACCENT_CYAN
        } else {
            widgets::ACCENT_CYAN.gamma_multiply(0.35)
        }
    } else if hovered {
        egui::Color32::from_rgb(36, 46, 66)
    } else {
        widgets::SURFACE_2
    };

    let border = if *on {
        widgets::ACCENT_CYAN
    } else if hovered {
        widgets::ACCENT_CYAN.gamma_multiply(0.7)
    } else {
        widgets::BORDER
    };

    painter.rect_filled(rect, CornerRadius::same(10), bg);
    painter.rect_stroke(
        rect,
        CornerRadius::same(10),
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );

    let thumb_x = if *on {
        rect.right() - 10.0
    } else {
        rect.left() + 10.0
    };
    let thumb_color = if enabled {
        egui::Color32::WHITE
    } else {
        widgets::TEXT_SUBTLE
    };
    painter.circle_filled(Pos2::new(thumb_x, rect.center().y), 7.0, thumb_color);

    response
}

/// Full-width row with strictly left-aligned label and right-aligned toggle switch.
pub fn toggle_row(ui: &mut egui::Ui, label: &str, val: &mut bool) {
    let row_w = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(row_w, ROW_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(label)
                        .size(13.0)
                        .color(widgets::TEXT_WEAK),
                )
                .truncate(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                toggle_switch(ui, val);
            });
        },
    );
}

fn apply_numeric_input_style(ui: &mut egui::Ui, size: egui::Vec2) {
    ui.style_mut().override_font_id = Some(FontId::monospace(12.0));
    ui.spacing_mut().interact_size = size;
    ui.spacing_mut().button_padding = egui::vec2(4.0, 1.0);

    let visuals = ui.visuals_mut();
    visuals.extreme_bg_color = widgets::SURFACE_2;
    visuals.override_text_color = Some(widgets::TEXT);

    let set_widget_style = |w: &mut egui::style::WidgetVisuals, border: egui::Color32| {
        w.bg_fill = widgets::SURFACE_2;
        w.corner_radius = CornerRadius::same(6);
        w.bg_stroke = Stroke::new(1.0, border);
        w.fg_stroke = Stroke::new(1.0, widgets::TEXT);
    };

    set_widget_style(&mut visuals.widgets.inactive, widgets::BORDER);
    set_widget_style(&mut visuals.widgets.hovered, widgets::ACCENT_CYAN);
    set_widget_style(&mut visuals.widgets.active, widgets::ACCENT_CYAN);
    set_widget_style(&mut visuals.widgets.open, widgets::ACCENT_CYAN);

    visuals.selection.bg_fill = widgets::ACCENT_CYAN.gamma_multiply(0.4);
    visuals.selection.stroke = Stroke::new(1.0, widgets::ACCENT_CYAN);
}

/// Interactive numeric input box for f32 values supporting direct keyboard typing and dragging.
pub fn numeric_input_f32(
    ui: &mut egui::Ui,
    val: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    speed: f64,
    decimals: usize,
    size: egui::Vec2,
) -> egui::Response {
    ui.scope(|ui| {
        apply_numeric_input_style(ui, size);
        let drag = egui::DragValue::new(val)
            .range(range)
            .speed(speed)
            .min_decimals(decimals.min(1))
            .max_decimals(decimals.max(1));
        ui.add_sized(size, drag)
    })
    .inner
}

/// Interactive numeric input box for u32 values supporting direct keyboard typing and dragging.
pub fn numeric_input_u32(
    ui: &mut egui::Ui,
    val: &mut u32,
    range: std::ops::RangeInclusive<u32>,
    size: egui::Vec2,
) -> egui::Response {
    ui.scope(|ui| {
        apply_numeric_input_style(ui, size);
        let drag = egui::DragValue::new(val).range(range).speed(1.0);
        ui.add_sized(size, drag)
    })
    .inner
}

/// Slider row with strictly left-aligned label, expanding cyan track, and flush-right value box.
pub fn slider_row(
    ui: &mut egui::Ui,
    label: &str,
    val: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    decimals: usize,
    fixed_slider_width: Option<f32>,
) {
    let row_w = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(row_w, ROW_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let label_w = if label.is_empty() {
                0.0
            } else {
                let w = (row_w * 0.28).clamp(74.0, 114.0);
                left_label(ui, label, w);
                w + 8.0
            };

            let input_w = 50.0;
            let slider_w = fixed_slider_width.unwrap_or_else(|| {
                (row_w - label_w - input_w - 8.0).max(48.0)
            });

            ui.spacing_mut().slider_width = slider_w;
            ui.visuals_mut().selection.bg_fill = widgets::ACCENT_CYAN;
            ui.visuals_mut().widgets.inactive.bg_fill = widgets::SURFACE_2;
            ui.visuals_mut().widgets.hovered.bg_fill = widgets::ACCENT_CYAN;
            ui.visuals_mut().widgets.active.bg_fill = widgets::ACCENT_CYAN;

            ui.add(egui::Slider::new(val, range.clone()).show_value(false));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                numeric_input_f32(ui, val, range, 0.1, decimals, egui::vec2(input_w, 22.0));
            });
        },
    );
}

/// Slider row for integer values with strictly left-aligned label, expanding track, and flush-right value box.
pub fn slider_row_u32(
    ui: &mut egui::Ui,
    label: &str,
    val: &mut u32,
    range: std::ops::RangeInclusive<u32>,
    fixed_slider_width: Option<f32>,
) {
    let row_w = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(row_w, ROW_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let label_w = if label.is_empty() {
                0.0
            } else {
                let w = (row_w * 0.28).clamp(74.0, 114.0);
                left_label(ui, label, w);
                w + 8.0
            };

            let input_w = 50.0;
            let slider_w = fixed_slider_width.unwrap_or_else(|| {
                (row_w - label_w - input_w - 8.0).max(48.0)
            });

            ui.spacing_mut().slider_width = slider_w;
            ui.visuals_mut().selection.bg_fill = widgets::ACCENT_CYAN;
            ui.visuals_mut().widgets.inactive.bg_fill = widgets::SURFACE_2;
            ui.visuals_mut().widgets.hovered.bg_fill = widgets::ACCENT_CYAN;
            ui.visuals_mut().widgets.active.bg_fill = widgets::ACCENT_CYAN;

            ui.add(egui::Slider::new(val, range.clone()).show_value(false));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                numeric_input_u32(ui, val, range, egui::vec2(input_w, 22.0));
            });
        },
    );
}

/// Sleek color swatch button.
pub fn color_swatch(ui: &mut egui::Ui, color: &mut [u8; 3], width: f32, height: f32) {
    ui.spacing_mut().interact_size = egui::vec2(width, height);
    ui.color_edit_button_srgb(color);
}

/// Top-right AUTO-SAVE ON pill indicator.
pub fn render_autosave_pill(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(120.0, 28.0), Sense::hover());
    let painter = ui.painter();

    let bg_color = egui::Color32::from_rgba_unmultiplied(16, 185, 129, 20);
    let border_color = egui::Color32::from_rgba_unmultiplied(16, 185, 129, 60);

    painter.rect_filled(rect, CornerRadius::same(14), bg_color);
    painter.rect_stroke(
        rect,
        CornerRadius::same(14),
        Stroke::new(1.0, border_color),
        StrokeKind::Inside,
    );

    let dot_center = Pos2::new(rect.min.x + 14.0, rect.center().y);
    painter.circle_filled(
        dot_center,
        4.5,
        egui::Color32::from_rgba_unmultiplied(16, 185, 129, 70),
    );
    painter.circle_filled(dot_center, 3.0, widgets::POSITIVE);

    painter.text(
        Pos2::new(rect.min.x + 24.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        "AUTO-SAVE ON",
        FontId::proportional(11.0),
        widgets::POSITIVE,
    );
}

/// Reset to defaults action pill button.
pub fn reset_defaults_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(190.0, 36.0), Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let hovered = response.hovered();

    let painter = ui.painter();
    let bg = if hovered {
        egui::Color32::from_rgb(26, 36, 56)
    } else {
        widgets::SURFACE_2
    };
    let border = if hovered {
        widgets::ACCENT_CYAN.gamma_multiply(0.8)
    } else {
        widgets::BORDER
    };

    painter.rect_filled(rect, CornerRadius::same(18), bg);
    painter.rect_stroke(rect, CornerRadius::same(18), Stroke::new(1.0, border), StrokeKind::Inside);

    let center_y = rect.center().y;
    let icon_c = Pos2::new(rect.min.x + 24.0, center_y);
    let text_pos = Pos2::new(rect.min.x + 40.0, center_y);

    let icon_col = if hovered { widgets::ACCENT_CYAN } else { widgets::TEXT_SUBTLE };
    draw_refresh_icon(painter, icon_c, 6.0, icon_col);

    let text_col = if hovered { egui::Color32::WHITE } else { widgets::TEXT };
    painter.text(
        text_pos,
        egui::Align2::LEFT_CENTER,
        "RESET TO DEFAULTS",
        FontId::proportional(11.5),
        text_col,
    );

    response
}

fn draw_refresh_icon(painter: &egui::Painter, c: Pos2, r: f32, col: egui::Color32) {
    let stroke = Stroke::new(1.5, col);
    let n = 12;
    for i in 0..n {
        let a0 = -std::f32::consts::FRAC_PI_2 + (i as f32 / n as f32) * (std::f32::consts::PI * 1.5);
        let a1 = -std::f32::consts::FRAC_PI_2 + ((i + 1) as f32 / n as f32) * (std::f32::consts::PI * 1.5);
        painter.line_segment(
            [
                c + Vec2::new(a0.cos() * r, a0.sin() * r),
                c + Vec2::new(a1.cos() * r, a1.sin() * r),
            ],
            stroke,
        );
    }
    let arrow_tip = c + Vec2::new(0.0, -r);
    painter.line_segment([arrow_tip, arrow_tip + Vec2::new(-3.0, -3.0)], stroke);
    painter.line_segment([arrow_tip, arrow_tip + Vec2::new(-3.0, 3.0)], stroke);
}
