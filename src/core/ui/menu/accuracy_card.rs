use bevy_egui::egui::{self, Color32, Pos2, Sense, Stroke, Vec2};

use super::chart;
use super::components::{self, CardIcon};
use super::stats::DashStats;
use crate::core::ui::widgets;

const LINE_WHITE: Color32 = Color32::from_rgb(236, 244, 252);

pub fn render_accuracy_card(ui: &mut egui::Ui, size: Vec2, stats: &DashStats) {
    widgets::fixed_card(ui, size, |ui| {
        let known: Vec<(usize, f32)> = stats
            .days
            .iter()
            .enumerate()
            .filter_map(|(i, d)| d.accuracy().map(|a| (i, a)))
            .collect();
        let shots: u32 = stats.days.iter().map(|d| d.shots).sum();
        let hits: u32 = stats.days.iter().map(|d| d.hits).sum();
        let overall = if shots > 0 {
            hits as f32 / shots as f32 * 100.0
        } else {
            0.0
        };
        let delta = match known.as_slice() {
            [.., (_, prev), (_, last)] => {
                Some((format!("{:.1}%", (last - prev).abs()), last >= prev))
            }
            _ => None,
        };

        components::card_header(
            ui,
            CardIcon::Target,
            widgets::ACCENT_CYAN,
            "Accuracy % Trend",
            Some("Last 7 days"),
            |ui| components::kpi(ui, &format!("{overall:.1}%"), delta),
        );
        ui.add_space(4.0);

        let area = Vec2::new(ui.available_width(), ui.available_height().max(60.0));
        let (rect, response) = ui.allocate_exact_size(area, Sense::hover());
        let painter = ui.painter_at(rect);
        let plot = chart::plot_rect(rect, 40.0);

        let x_labels: Vec<String> = stats.days.iter().map(|d| d.label.clone()).collect();
        let y_labels = chart::y_scale(100.0, 5, |v| format!("{}%", v.round() as i64));
        chart::grid(&painter, plot, &y_labels, &x_labels, true);

        let cols = (stats.days.len() - 1).max(1) as f32;
        let pts: Vec<Pos2> = known
            .iter()
            .map(|(i, a)| {
                Pos2::new(
                    plot.left() + plot.width() * *i as f32 / cols,
                    plot.bottom() - plot.height() * (a / 100.0).clamp(0.0, 1.0),
                )
            })
            .collect();
        if pts.len() >= 2 {
            let curve: Vec<Pos2> = chart::smooth(&pts, 8)
                .into_iter()
                .map(|p| Pos2::new(p.x, p.y.clamp(plot.top(), plot.bottom())))
                .collect();
            chart::area_fill(
                &painter,
                &curve,
                plot.bottom(),
                Color32::from_rgba_unmultiplied(56, 188, 248, 42),
                Color32::from_rgba_unmultiplied(56, 188, 248, 0),
            );
            chart::glow_line(&painter, &curve, LINE_WHITE, 2.0);
        }

        let hovered_idx = response.hover_pos().and_then(|m| {
            if !plot.expand2(Vec2::new(10.0, 20.0)).contains(m) || pts.is_empty() {
                return None;
            }
            pts.iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    (a.x - m.x).abs().total_cmp(&(b.x - m.x).abs())
                })
                .map(|(idx, _)| idx)
        });

        for (k, p) in pts.iter().enumerate() {
            let active = hovered_idx == Some(k) || (hovered_idx.is_none() && k + 1 == pts.len());
            let r = if active { 4.5 } else { 3.0 };
            painter.circle_filled(*p, r, if active { Color32::WHITE } else { widgets::ACCENT_CYAN });
            painter.circle_stroke(*p, r, Stroke::new(1.5, widgets::ACCENT_CYAN));
        }

        if let Some(idx) = hovered_idx {
            let (_, acc) = known[idx];
            chart::node_with_tooltip(&painter, pts[idx], rect, &format!("{acc:.1}%"), None);
        }
    });
}
