use bevy_egui::egui::{self, Color32, Pos2, Sense, Stroke, Vec2};

use super::components::{ResultsIcon, card_header};
use crate::core::storage::TaskScores;
use crate::core::ui::menu::chart;
use crate::core::ui::widgets;

/// Point graph displaying the last 5 recorded accuracy percentages for the active task.
pub fn render_accuracy_chart_card(
    ui: &mut egui::Ui,
    size: Vec2,
    task_scores: Option<&TaskScores>,
) {
    widgets::fixed_card(ui, size, |ui| {
        let values: Vec<f32> = task_scores
            .map(|t| {
                let count = t.recent.len().min(5);
                t.recent[t.recent.len() - count..]
                    .iter()
                    .map(|r| {
                        let total = r.hits + r.misses;
                        if total > 0 {
                            (r.hits as f32 / total as f32 * 100.0).clamp(0.0, 100.0)
                        } else {
                            0.0
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let latest = values.last().copied().unwrap_or(0.0);
        let latest_str = format!("{latest:.1}%");

        card_header(
            ui,
            ResultsIcon::Crosshair,
            widgets::TARGET_YELLOW,
            "Last 5 Accuracy %",
            Some("Hit ratio per round"),
            Some((&latest_str, widgets::TARGET_YELLOW)),
        );
        ui.add_space(4.0);

        let area = Vec2::new(ui.available_width(), ui.available_height().max(70.0));
        let (rect, response) = ui.allocate_exact_size(area, Sense::hover());
        let painter = ui.painter_at(rect);

        if values.is_empty() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "No accuracy history yet",
                egui::FontId::proportional(12.0),
                widgets::TEXT_WEAK,
            );
            return;
        }

        let n = values.len();
        let plot = chart::plot_rect(rect, 44.0);
        let max = 100.0;

        let x_labels: Vec<String> = (0..n).map(|i| format!("R{}", i + 1)).collect();
        let y_labels = chart::y_scale(max, 4, |v| format!("{}%", v.round() as i64));
        chart::grid(&painter, plot, &y_labels, &x_labels, true);

        let pts: Vec<Pos2> = values
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let x = if n == 1 {
                    plot.center().x
                } else {
                    plot.left() + plot.width() * i as f32 / (n - 1) as f32
                };
                Pos2::new(x, plot.bottom() - plot.height() * (v / max).clamp(0.0, 1.0))
            })
            .collect();

        if n >= 2 {
            let curve: Vec<Pos2> = chart::smooth(&pts, 8)
                .into_iter()
                .map(|p| Pos2::new(p.x, p.y.clamp(plot.top(), plot.bottom())))
                .collect();
            chart::area_fill(
                &painter,
                &curve,
                plot.bottom(),
                Color32::from_rgba_unmultiplied(255, 214, 10, 64),
                Color32::from_rgba_unmultiplied(255, 214, 10, 0),
            );
            chart::glow_line(&painter, &curve, widgets::TARGET_YELLOW, 2.0);
        }

        for (i, p) in pts.iter().enumerate() {
            let is_last = i + 1 == n;
            let r = if is_last { 5.0 } else { 4.0 };
            painter.circle_filled(
                *p,
                r,
                if is_last { Color32::WHITE } else { widgets::TARGET_YELLOW },
            );
            painter.circle_stroke(*p, r, Stroke::new(1.5, widgets::TARGET_YELLOW_DIM));
        }

        let focus = if n == 1 {
            0
        } else {
            match response.hover_pos() {
                Some(m) if plot.expand2(Vec2::new(10.0, 24.0)).contains(m) => {
                    let frac = ((m.x - plot.left()) / plot.width()).clamp(0.0, 1.0);
                    ((frac * (n - 1) as f32).round() as usize).min(n - 1)
                }
                _ => n - 1,
            }
        };

        let val = values[focus];
        let delta = (focus > 0).then(|| {
            let diff = val - values[focus - 1];
            (format!("{diff:+.1}%"), diff >= 0.0)
        });
        chart::node_with_tooltip(&painter, pts[focus], rect, &format!("{val:.1}%"), delta);
    });
}
