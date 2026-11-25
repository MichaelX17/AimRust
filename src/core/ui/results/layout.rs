use bevy_egui::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Layout, Pos2, Rect, Sense, Stroke,
    StrokeKind, Vec2,
};

use crate::core::state::AppState;
use crate::core::ui::widgets;

pub const ACTIONS_HEIGHT: f32 = 46.0;
pub const SECTION_GAP: f32 = 14.0;

pub fn render_twin_row(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    gap: f32,
    render_left: impl FnOnce(&mut egui::Ui, Vec2),
    render_right: impl FnOnce(&mut egui::Ui, Vec2),
) {
    let card_w = (width - gap) * 0.5;
    let (row_area, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let left_rect = Rect::from_min_size(row_area.min, Vec2::new(card_w, height));
    let right_rect = Rect::from_min_size(
        Pos2::new(left_rect.right() + gap, row_area.top()),
        Vec2::new(card_w, height),
    );

    let mut left_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(left_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    render_left(&mut left_ui, Vec2::new(card_w, height));

    let mut right_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(right_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    render_right(&mut right_ui, Vec2::new(card_w, height));
}

/// Stylized Play Again & Go Menu action bar matching the dashboard & modal aesthetic.
pub fn render_actions(
    ui: &mut egui::Ui,
    width: f32,
    next_state: &mut bevy::prelude::NextState<AppState>,
) {
    let gap = 14.0;
    let btn_w = ((width - gap) * 0.5).clamp(150.0, 260.0);
    let total_w = btn_w * 2.0 + gap;
    let side_pad = ((width - total_w) * 0.5).max(0.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        if side_pad > 0.0 {
            ui.add_space(side_pad);
        }

        if menu_action_button(ui, Vec2::new(btn_w, ACTIONS_HEIGHT)).clicked() {
            next_state.set(AppState::Menu);
        }

        ui.add_space(gap);

        if play_again_button(ui, Vec2::new(btn_w, ACTIONS_HEIGHT)).clicked() {
            next_state.set(AppState::Countdown);
        }
    });
}

fn play_again_button(ui: &mut egui::Ui, size: Vec2) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let fill = if response.is_pointer_button_down_on() {
        widgets::TARGET_YELLOW_DIM
    } else if response.hovered() {
        Color32::from_rgb(255, 226, 70)
    } else {
        widgets::TARGET_YELLOW
    };

    let p = ui.painter_at(rect);
    p.rect_filled(rect, CornerRadius::same(10), fill);

    let fg = widgets::BG;
    let label_galley = ui
        .painter()
        .layout_no_wrap("PLAY AGAIN".to_string(), FontId::proportional(14.0), fg);
    let content_w = 18.0 + label_galley.size().x + 56.0;
    let start_x = rect.center().x - content_w * 0.5;
    let cy = rect.center().y;

    // Play triangle icon
    let ic = Pos2::new(start_x + 6.0, cy);
    p.add(egui::Shape::convex_polygon(
        vec![
            ic + Vec2::new(-4.0, -5.5),
            ic + Vec2::new(5.5, 0.0),
            ic + Vec2::new(-4.0, 5.5),
        ],
        fg,
        Stroke::NONE,
    ));

    p.galley(
        Pos2::new(start_x + 18.0, cy - label_galley.size().y * 0.5),
        label_galley,
        fg,
    );

    // Subtle SPACE keycap hint
    let key_rect = Rect::from_center_size(Pos2::new(start_x + content_w - 22.0, cy), Vec2::new(44.0, 20.0));
    p.rect_filled(key_rect, CornerRadius::same(4), fg.gamma_multiply(0.14));
    p.text(key_rect.center(), Align2::CENTER_CENTER, "SPACE", FontId::monospace(9.5), fg);

    response
}

fn menu_action_button(ui: &mut egui::Ui, size: Vec2) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let (fill, border) = if response.hovered() {
        (widgets::SURFACE_HOVER, Color32::from_rgb(76, 86, 102))
    } else {
        (widgets::SURFACE_2, widgets::BORDER)
    };

    let p = ui.painter_at(rect);
    p.rect_filled(rect, CornerRadius::same(10), fill);
    p.rect_stroke(rect, CornerRadius::same(10), Stroke::new(1.0, border), StrokeKind::Inside);

    let fg = widgets::TEXT;
    let label_galley = ui
        .painter()
        .layout_no_wrap("GO MENU".to_string(), FontId::proportional(14.0), fg);
    let content_w = 20.0 + label_galley.size().x + 44.0;
    let start_x = rect.center().x - content_w * 0.5;
    let cy = rect.center().y;

    // 4-tile dashboard grid icon
    let ic = Pos2::new(start_x + 6.0, cy);
    for dx in [-4.5, 1.0] {
        for dy in [-4.5, 1.0] {
            p.rect_filled(
                Rect::from_min_size(ic + Vec2::new(dx, dy), Vec2::splat(3.5)),
                CornerRadius::same(1),
                widgets::TEXT_WEAK,
            );
        }
    }

    p.galley(
        Pos2::new(start_x + 20.0, cy - label_galley.size().y * 0.5),
        label_galley,
        fg,
    );

    // Subtle ESC keycap hint
    let key_rect = Rect::from_center_size(Pos2::new(start_x + content_w - 16.0, cy), Vec2::new(32.0, 20.0));
    p.rect_filled(key_rect, CornerRadius::same(4), widgets::SURFACE);
    p.rect_stroke(key_rect, CornerRadius::same(4), Stroke::new(1.0, widgets::BORDER), StrokeKind::Inside);
    p.text(
        key_rect.center(),
        Align2::CENTER_CENTER,
        "ESC",
        FontId::monospace(9.5),
        widgets::TEXT_WEAK,
    );

    response
}
