use bevy_egui::egui::{self, Color32, Pos2, Rect, Sense, Vec2};

use super::chart;
use super::components::{self, CardIcon};
use super::stats::DashStats;
use crate::core::ui::widgets;

const REPS_WHITE: Color32 = Color32::from_rgb(232, 240, 250);

/// Saturated target yellow (#FFD60A / #F5B700) = targets destroyed (hits),
/// crisp white = daily reps (shots fired).
pub fn render_targets_card(ui: &mut egui::Ui, size: Vec2, stats: &DashStats) {
    widgets::fixed_card(ui, size, |ui| {
        let total: u32 = stats.days.iter().map(|d| d.hits).sum();
        let subtitle = if size.x < 360.0 {
            "7 days"
        } else {
            "Daily Reps · Last 7 days"
        };
        components::card_header(
            ui,
            CardIcon::Target,
            widgets::TARGET_YELLOW,
            "Targets Destroyed",
            Some(subtitle),
            |ui| components::kpi(ui, &widgets::thousands(total as i64), None),
        );
        ui.horizontal(|ui| {
            components::legend_item(ui, widgets::TARGET_YELLOW, "Targets");
            components::legend_item(ui, REPS_WHITE, "Reps");
        });
        ui.add_space(2.0);

        let area = Vec2::new(ui.available_width(), ui.available_height().max(60.0));
        let (rect, response) = ui.allocate_exact_size(area, Sense::hover());
        let painter = ui.painter_at(rect);
        let plot = chart::plot_rect(rect, 40.0);

        let peak = stats
            .days
            .iter()
            .map(|d| d.shots.max(d.hits))
            .max()
            .unwrap_or(0) as f32;
        let max = chart::nice_max(peak * 1.08);
        let x_labels: Vec<String> = stats.days.iter().map(|d| d.label.clone()).collect();
        let y_labels = chart::y_scale(max, 5, chart::axis_value);

        // Bars sit centered in 7 equal slots; labels use the same slots.
        let slots = stats.days.len().max(1) as f32;
        let slot_w = plot.width() / slots;
        let grid_plot = Rect::from_min_max(
            Pos2::new(plot.left() + slot_w * 0.5, plot.top()),
            Pos2::new(plot.right() - slot_w * 0.5, plot.bottom()),
        );
        chart::grid(&painter, plot, &y_labels, &[], false);
        for (i, label) in x_labels.iter().enumerate() {
            painter.text(
                Pos2::new(grid_plot.left() + slot_w * i as f32, plot.bottom() + 5.0),
                egui::Align2::CENTER_TOP,
                label,
                egui::FontId::monospace(10.5),
                widgets::TEXT_SUBTLE,
            );
        }

        let hovered_slot = response.hover_pos().and_then(|m| {
            if !plot.contains(m) {
                return None;
            }
            let idx = ((m.x - plot.left()) / slot_w).floor() as isize;
            if idx >= 0 && (idx as usize) < stats.days.len() {
                Some(idx as usize)
            } else {
                None
            }
        });

        let bar_w = (slot_w * 0.16).clamp(4.0, 8.0);
        let gap = 3.0;
        let radius = (bar_w * 0.5).round() as u8;
        for (i, d) in stats.days.iter().enumerate() {
            let cx = grid_plot.left() + slot_w * i as f32;
            let yellow = if hovered_slot == Some(i) {
                widgets::TARGET_YELLOW
            } else {
                widgets::TARGET_YELLOW_DIM
            };
            for (k, (value, color)) in [(d.shots, REPS_WHITE), (d.hits, yellow)]
                .into_iter()
                .enumerate()
            {
                let h = plot.height() * (value as f32 / max).clamp(0.0, 1.0);
                if h < 1.0 {
                    continue;
                }
                let x = if k == 0 {
                    cx - gap * 0.5 - bar_w
                } else {
                    cx + gap * 0.5
                };
                let bar = Rect::from_min_max(
                    Pos2::new(x, plot.bottom() - h),
                    Pos2::new(x + bar_w, plot.bottom()),
                );
                painter.rect_filled(
                    bar,
                    egui::CornerRadius {
                        nw: radius,
                        ne: radius,
                        sw: 0,
                        se: 0,
                    },
                    color,
                );
            }
        }

        if let Some(i) = hovered_slot {
            let d = &stats.days[i];
            let cx = grid_plot.left() + slot_w * i as f32;
            let top_h = plot.height() * (d.shots.max(d.hits) as f32 / max).clamp(0.0, 1.0);
            let node = Pos2::new(cx, plot.bottom() - top_h);
            chart::node_with_tooltip(
                &painter,
                node,
                rect,
                &format!("{} / {}", d.hits, d.shots),
                None,
            );
        }
    });
}
