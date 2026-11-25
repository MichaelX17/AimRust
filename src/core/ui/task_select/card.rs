use bevy_egui::egui::{self, Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, Vec2};

use crate::core::storage::Scores;
use crate::core::task_api::{ScoringMode, TaskInfo};
use crate::core::ui::widgets;

pub fn render_task_card(
    ui: &mut egui::Ui,
    task: &TaskInfo,
    selected: bool,
    scores: &Scores,
    accent: Color32,
    size: Vec2,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let is_hovered = response.hovered();
    let is_clicked = response.clicked();
    let p = ui.painter_at(rect);

    // Surface & Border with hover / selected state
    let bg_color = if selected {
        widgets::SURFACE_2
    } else if is_hovered {
        Color32::from_rgb(22, 32, 53)
    } else {
        widgets::SURFACE
    };

    let border_color = if selected {
        accent
    } else if is_hovered {
        widgets::BORDER.lerp_to_gamma(accent, 0.45)
    } else {
        widgets::BORDER
    };

    let border_width = if selected { 1.5 } else { 1.0 };

    p.rect_filled(rect, CornerRadius::same(12), bg_color);
    p.rect_stroke(
        rect,
        CornerRadius::same(12),
        Stroke::new(border_width, border_color),
        egui::StrokeKind::Inside,
    );

    let inner = rect.shrink(14.0);

    // Mode tag badge on top right
    let (mode_text, mode_color) = match task.scoring {
        ScoringMode::Click => ("CLICK", widgets::ACCENT_CYAN),
        ScoringMode::Tracking => ("TRACKING", widgets::ACCENT_VIOLET),
    };

    let tag_size = Vec2::new(72.0, 20.0);
    let tag_rect = Rect::from_min_size(
        Pos2::new(inner.right() - tag_size.x, inner.top()),
        tag_size,
    );
    p.rect_filled(tag_rect, CornerRadius::same(5), mode_color.gamma_multiply(0.16));
    p.rect_stroke(
        tag_rect,
        CornerRadius::same(5),
        Stroke::new(1.0, mode_color.gamma_multiply(0.5)),
        egui::StrokeKind::Inside,
    );
    p.text(
        tag_rect.center(),
        Align2::CENTER_CENTER,
        mode_text,
        FontId::monospace(10.0),
        mode_color,
    );

    // Sniper mode indicator if applicable
    if task.is_sniper_mode {
        let sniper_tag = Rect::from_min_size(
            Pos2::new(tag_rect.left() - 62.0, inner.top()),
            Vec2::new(56.0, 20.0),
        );
        p.rect_filled(sniper_tag, CornerRadius::same(5), widgets::TARGET_YELLOW.gamma_multiply(0.16));
        p.rect_stroke(
            sniper_tag,
            CornerRadius::same(5),
            Stroke::new(1.0, widgets::TARGET_YELLOW.gamma_multiply(0.5)),
            egui::StrokeKind::Inside,
        );
        p.text(
            sniper_tag.center(),
            Align2::CENTER_CENTER,
            "SNIPER",
            FontId::monospace(9.5),
            widgets::TARGET_YELLOW,
        );
    }

    // Task Title
    p.text(
        inner.left_top(),
        Align2::LEFT_TOP,
        task.name,
        FontId::proportional(16.0),
        if selected { widgets::TEXT } else { Color32::WHITE },
    );

    // Task Description (clipped to available width)
    let desc_rect = Rect::from_min_size(
        Pos2::new(inner.left(), inner.top() + 26.0),
        Vec2::new(inner.width(), 36.0),
    );
    p.text(
        desc_rect.left_top(),
        Align2::LEFT_TOP,
        task.description,
        FontId::proportional(11.5),
        widgets::TEXT_SUBTLE,
    );

    // Bottom row: High score & Play action
    let best_score = scores
        .tasks
        .get(task.id)
        .and_then(|t| t.best.values().map(|b| b.score).max());

    let bottom_y = inner.bottom() - 10.0;
    let best_text = match best_score {
        Some(s) => format!("PB: {} pts", widgets::thousands(s as i64)),
        None => "PB: —".to_string(),
    };

    p.text(
        Pos2::new(inner.left(), bottom_y),
        Align2::LEFT_CENTER,
        best_text,
        FontId::monospace(11.5),
        if best_score.is_some() { widgets::ACCENT_CYAN_GLOW } else { widgets::TEXT_WEAK },
    );

    // Arrow / Select affordance
    let arrow_color = if selected {
        accent
    } else if is_hovered {
        widgets::TEXT
    } else {
        widgets::TEXT_WEAK
    };

    p.text(
        Pos2::new(inner.right(), bottom_y),
        Align2::RIGHT_CENTER,
        if selected { "SELECTED  ✓" } else { "SELECT  →" },
        FontId::proportional(11.0),
        arrow_color,
    );

    is_clicked
}
