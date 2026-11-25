//! Reusable chart primitives: smoothing, gradient fills, glow lines, grid, tooltip.

use bevy_egui::egui::{self, Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};

use crate::core::ui::widgets;

/// Plot area inside a chart rect (room for y labels left, x labels below).
pub fn plot_rect(rect: Rect, left_pad: f32) -> Rect {
    Rect::from_min_max(
        Pos2::new(rect.left() + left_pad, rect.top() + 6.0),
        Pos2::new(rect.right() - 8.0, rect.bottom() - 20.0),
    )
}

/// Catmull-Rom spline through `pts`, `steps` samples per segment.
pub fn smooth(pts: &[Pos2], steps: usize) -> Vec<Pos2> {
    let n = pts.len();
    if n < 3 {
        return pts.to_vec();
    }
    let mut out = Vec::with_capacity(n * steps);
    for i in 0..n - 1 {
        let p0 = pts[i.saturating_sub(1)];
        let (p1, p2) = (pts[i], pts[i + 1]);
        let p3 = pts[(i + 2).min(n - 1)];
        for s in 0..steps {
            let t = s as f32 / steps as f32;
            let (t2, t3) = (t * t, t * t * t);
            let f = |a: f32, b: f32, c: f32, d: f32| {
                0.5 * (2.0 * b
                    + (-a + c) * t
                    + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2
                    + (-a + 3.0 * b - 3.0 * c + d) * t3)
            };
            out.push(Pos2::new(f(p0.x, p1.x, p2.x, p3.x), f(p0.y, p1.y, p2.y, p3.y)));
        }
    }
    out.push(pts[n - 1]);
    out
}

/// Vertical gradient under a curve, from `top` (at the curve) to `bottom` (baseline).
pub fn area_fill(painter: &Painter, curve: &[Pos2], baseline: f32, top: Color32, bottom: Color32) {
    let mut mesh = egui::Mesh::default();
    for (i, p) in curve.iter().enumerate() {
        mesh.colored_vertex(*p, top);
        mesh.colored_vertex(Pos2::new(p.x, baseline), bottom);
        if i > 0 {
            let b = ((i - 1) * 2) as u32;
            mesh.add_triangle(b, b + 1, b + 2);
            mesh.add_triangle(b + 1, b + 3, b + 2);
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// Curve with a soft glow underneath.
pub fn glow_line(painter: &Painter, curve: &[Pos2], color: Color32, width: f32) {
    painter.add(egui::Shape::line(
        curve.to_vec(),
        Stroke::new(width + 4.0, color.gamma_multiply(0.14)),
    ));
    painter.add(egui::Shape::line(curve.to_vec(), Stroke::new(width, color)));
}

/// Horizontal gridlines with right-aligned y labels (top to bottom), optional
/// vertical gridlines and centered x labels.
pub fn grid(painter: &Painter, plot: Rect, y_labels: &[String], x_labels: &[String], vertical: bool) {
    let label_color = widgets::TEXT_SUBTLE;
    let rows = y_labels.len().max(2) - 1;
    for (i, label) in y_labels.iter().enumerate() {
        let y = plot.top() + plot.height() * i as f32 / rows as f32;
        painter.line_segment(
            [Pos2::new(plot.left(), y), Pos2::new(plot.right(), y)],
            Stroke::new(1.0, widgets::GRID),
        );
        painter.text(
            Pos2::new(plot.left() - 8.0, y),
            Align2::RIGHT_CENTER,
            label,
            FontId::monospace(10.5),
            label_color,
        );
    }
    let cols = x_labels.len().max(2) - 1;
    for (i, label) in x_labels.iter().enumerate() {
        let x = plot.left() + plot.width() * i as f32 / cols as f32;
        if vertical {
            painter.line_segment(
                [Pos2::new(x, plot.top()), Pos2::new(x, plot.bottom())],
                Stroke::new(1.0, widgets::GRID.gamma_multiply(0.7)),
            );
        }
        painter.text(
            Pos2::new(x, plot.bottom() + 5.0),
            Align2::CENTER_TOP,
            label,
            FontId::monospace(10.5),
            label_color,
        );
    }
}

/// Round axis maximum up to a "nice" value (1, 1.5, 2, 2.5, 3, 4, 5, 7.5 × 10^k).
pub fn nice_max(value: f32) -> f32 {
    let value = value.max(1.0);
    let mag = 10f32.powf(value.log10().floor());
    for f in [1.0, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0, 7.5, 10.0] {
        if value <= f * mag {
            return f * mag;
        }
    }
    10.0 * mag
}

/// 100000 -> "100k", 2500 -> "2.5k", 750 -> "750".
pub fn axis_value(value: f32) -> String {
    if value >= 1000.0 {
        let k = value / 1000.0;
        if (k - k.round()).abs() < 0.05 {
            format!("{}k", k.round() as i64)
        } else {
            format!("{k:.1}k")
        }
    } else {
        format!("{}", value.round() as i64)
    }
}

/// `count` labels from `max` down to 0.
pub fn y_scale(max: f32, count: usize, fmt: impl Fn(f32) -> String) -> Vec<String> {
    let rows = count.max(2) - 1;
    (0..count).map(|i| fmt(max * (1.0 - i as f32 / rows as f32))).collect()
}

/// Glowing node + cyan-bordered tooltip pill centered above it.
pub fn node_with_tooltip(painter: &Painter, node: Pos2, bounds: Rect, text: &str, delta: Option<(String, bool)>) {
    painter.line_segment(
        [node, Pos2::new(node.x, bounds.bottom())],
        Stroke::new(1.0, widgets::ACCENT_CYAN.gamma_multiply(0.45)),
    );
    painter.circle_filled(node, 8.0, widgets::ACCENT_CYAN_GLOW.gamma_multiply(0.25));
    painter.circle_filled(node, 4.5, Color32::WHITE);
    painter.circle_stroke(node, 4.5, Stroke::new(1.5, widgets::ACCENT_CYAN));

    let size = Vec2::new(88.0, if delta.is_some() { 40.0 } else { 24.0 });
    let half = size.x * 0.5;
    // Keep the pill inside the chart horizontally while the tip stays on the node.
    let cx = node.x.clamp(bounds.left() + half, bounds.right() - half);
    let bottom = (node.y - 12.0).max(bounds.top() + size.y);
    let pill = Rect::from_min_size(Pos2::new(cx - half, bottom - size.y), size);
    let fill = widgets::SURFACE_HOVER;
    let border = Stroke::new(1.0, Color32::from_rgb(76, 86, 102));
    painter.rect_filled(pill, egui::CornerRadius::same(7), fill);
    painter.rect_stroke(pill, egui::CornerRadius::same(7), border, egui::StrokeKind::Inside);
    painter.add(egui::Shape::convex_polygon(
        vec![
            Pos2::new(node.x - 5.0, pill.bottom() - 1.0),
            Pos2::new(node.x + 5.0, pill.bottom() - 1.0),
            Pos2::new(node.x, pill.bottom() + 5.0),
        ],
        fill,
        border,
    ));
    let text_y = if delta.is_some() { pill.top() + 12.5 } else { pill.center().y };
    painter.text(
        Pos2::new(pill.center().x, text_y),
        Align2::CENTER_CENTER,
        text,
        FontId::monospace(12.5),
        Color32::WHITE,
    );
    if let Some((delta, positive)) = delta {
        let color = if positive { widgets::POSITIVE } else { widgets::NEGATIVE };
        painter.text(
            Pos2::new(pill.center().x, pill.bottom() - 11.5),
            Align2::CENTER_CENTER,
            delta,
            FontId::monospace(10.5),
            color,
        );
    }
}
