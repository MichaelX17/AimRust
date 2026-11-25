use bevy_egui::egui::{self, CornerRadius, Pos2, Rect, Stroke, StrokeKind};

use crate::core::ui::widgets;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingsCategory {
    Aim,
    Crosshair,
    Environment,
    TargetUi,
    Video,
    Gameplay,
    Audio,
}

/// Draws clean procedural vector icons for each category.
pub fn draw_category_icon(painter: &egui::Painter, rect: Rect, category: SettingsCategory) {
    let c = rect.center();
    let cyan = widgets::ACCENT_CYAN;
    let stroke = Stroke::new(1.5, cyan);

    match category {
        SettingsCategory::Aim => {
            // Reticle / Aim circle with outer tick marks and center dot
            painter.circle_stroke(c, 7.0, stroke);
            painter.circle_filled(c, 1.5, cyan);
            painter.line_segment([c - egui::vec2(9.5, 0.0), c - egui::vec2(7.0, 0.0)], stroke);
            painter.line_segment([c + egui::vec2(7.0, 0.0), c + egui::vec2(9.5, 0.0)], stroke);
            painter.line_segment([c - egui::vec2(0.0, 9.5), c - egui::vec2(0.0, 7.0)], stroke);
            painter.line_segment([c + egui::vec2(0.0, 7.0), c + egui::vec2(0.0, 9.5)], stroke);
        }
        SettingsCategory::Crosshair => {
            // Distinct crosshair with 4 ticks and central gap
            painter.line_segment([c - egui::vec2(8.0, 0.0), c - egui::vec2(3.0, 0.0)], stroke);
            painter.line_segment([c + egui::vec2(3.0, 0.0), c + egui::vec2(8.0, 0.0)], stroke);
            painter.line_segment([c - egui::vec2(0.0, 8.0), c - egui::vec2(0.0, 3.0)], stroke);
            painter.line_segment([c + egui::vec2(0.0, 3.0), c + egui::vec2(0.0, 8.0)], stroke);
            painter.circle_filled(c, 1.0, cyan);
        }
        SettingsCategory::Environment => {
            // Stylized leaf icon
            let p0 = c + egui::vec2(-6.0, 6.0);
            let p1 = c + egui::vec2(6.0, -6.0);
            painter.line_segment([p0, p1], stroke);
            // Leaf arcs
            let mut top_pts = Vec::new();
            let mut bot_pts = Vec::new();
            for i in 0..=8 {
                let t = i as f32 / 8.0;
                let offset_t = (t * std::f32::consts::PI).sin() * 4.5;
                top_pts.push(Pos2::new(
                    p0.x + (p1.x - p0.x) * t - offset_t * 0.7,
                    p0.y + (p1.y - p0.y) * t - offset_t * 0.7,
                ));
                bot_pts.push(Pos2::new(
                    p0.x + (p1.x - p0.x) * t + offset_t * 0.7,
                    p0.y + (p1.y - p0.y) * t + offset_t * 0.7,
                ));
            }
            for i in 0..top_pts.len() - 1 {
                painter.line_segment([top_pts[i], top_pts[i + 1]], Stroke::new(1.2, cyan));
                painter.line_segment([bot_pts[i], bot_pts[i + 1]], Stroke::new(1.2, cyan));
            }
        }
        SettingsCategory::TargetUi => {
            // Concentric target circles
            painter.circle_stroke(c, 7.5, Stroke::new(1.2, cyan));
            painter.circle_stroke(c, 4.0, Stroke::new(1.2, cyan));
            painter.circle_filled(c, 1.8, cyan);
        }
        SettingsCategory::Video => {
            // Monitor screen and stand
            let screen_rect = Rect::from_center_size(c - egui::vec2(0.0, 2.0), egui::vec2(16.0, 11.0));
            painter.rect_stroke(screen_rect, CornerRadius::same(2), stroke, StrokeKind::Inside);
            // Stand
            painter.line_segment([c + egui::vec2(0.0, 3.5), c + egui::vec2(0.0, 7.0)], stroke);
            painter.line_segment([c + egui::vec2(-4.0, 7.0), c + egui::vec2(4.0, 7.0)], stroke);
        }
        SettingsCategory::Gameplay => {
            // Gamepad controller
            let pad_rect = Rect::from_center_size(c, egui::vec2(17.0, 11.0));
            painter.rect_stroke(pad_rect, CornerRadius::same(4), stroke, StrokeKind::Inside);
            // D-pad left
            let d_c = c - egui::vec2(4.0, 0.0);
            painter.line_segment([d_c - egui::vec2(2.0, 0.0), d_c + egui::vec2(2.0, 0.0)], Stroke::new(1.2, cyan));
            painter.line_segment([d_c - egui::vec2(0.0, 2.0), d_c + egui::vec2(0.0, 2.0)], Stroke::new(1.2, cyan));
            // Action buttons right
            painter.circle_filled(c + egui::vec2(3.5, -1.5), 1.0, cyan);
            painter.circle_filled(c + egui::vec2(5.5, 1.0), 1.0, cyan);
        }
        SettingsCategory::Audio => {
            // Speaker cone + sound waves
            let body = [
                c + egui::vec2(-7.5, -2.5),
                c + egui::vec2(-4.0, -2.5),
                c + egui::vec2(0.0, -6.0),
                c + egui::vec2(0.0, 6.0),
                c + egui::vec2(-4.0, 2.5),
                c + egui::vec2(-7.5, 2.5),
            ];
            for i in 0..body.len() {
                painter.line_segment([body[i], body[(i + 1) % body.len()]], stroke);
            }
            // Inner & outer wave arcs approximated with short segments
            for &(r, spread) in &[(3.2f32, 0.7f32), (6.0f32, 0.85f32)] {
                let mut prev: Option<Pos2> = None;
                for step in 0..=6 {
                    let a = -spread + (2.0 * spread) * (step as f32 / 6.0);
                    let pt = c + egui::vec2(1.5 + r * a.cos(), r * a.sin());
                    if let Some(p) = prev {
                        painter.line_segment([p, pt], Stroke::new(1.3, cyan));
                    }
                    prev = Some(pt);
                }
            }
        }
    }
}
