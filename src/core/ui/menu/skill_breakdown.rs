use std::f32::consts::{FRAC_PI_2, TAU};

use bevy_egui::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

use super::components::{self, CardIcon};
use super::stats::{DAILY_GOAL_HITS, DashStats};
use crate::core::ui::widgets;

const TILE_H: f32 = 52.0;
const TILE_GAP: f32 = 8.0;
const RING: [Color32; 4] = [
    Color32::from_rgb(218, 232, 245), // crisp ice-white
    widgets::TARGET_YELLOW,           // #FFD60A saturated target yellow
    widgets::TARGET_YELLOW_DIM,       // #F5B700 warm amber
    Color32::from_rgb(108, 120, 142), // cool slate
];

pub fn render_skill_breakdown(ui: &mut egui::Ui, size: Vec2, stats: &DashStats) {
    widgets::fixed_card(ui, size, |ui| {
        components::card_header(
            ui,
            CardIcon::Bars,
            widgets::TARGET_YELLOW,
            "Skill Breakdown",
            None,
            |ui| {
                ui.label(
                    egui::RichText::new("Session Telemetry")
                        .size(11.5)
                        .color(widgets::TEXT_SUBTLE),
                );
            },
        );
        ui.add_space(4.0);

        // Donut takes whatever the 2x2 stat grid leaves free.
        let tiles_h = TILE_H * 2.0 + TILE_GAP;
        let donut_h = (ui.available_height() - tiles_h - 10.0).clamp(80.0, 200.0);
        let (donut, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), donut_h), Sense::hover());
        draw_donut(ui, donut, stats);
        ui.add_space(8.0);

        let (area, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), tiles_h), Sense::hover());
        let w = (area.width() - TILE_GAP) * 0.5;
        let avg = stats
            .avg_ms_per_hit
            .map_or("—".to_string(), |ms| format!("{ms:.0} ms"));
        let goal_met = stats.hits_today >= DAILY_GOAL_HITS;
        let tiles = [
            ("Avg TTK", avg, RING[0], false),
            (
                "Hits vs. Misses",
                format!("{} / {}", stats.hits, stats.misses),
                widgets::TARGET_YELLOW,
                false,
            ),
            (
                "Best Streak",
                format!("{} hits", stats.best_round_hits),
                widgets::POSITIVE,
                false,
            ),
            (
                "Daily Goal",
                format!(
                    "{}/{}",
                    stats.hits_today.min(DAILY_GOAL_HITS),
                    DAILY_GOAL_HITS
                ),
                widgets::ACCENT_CYAN,
                goal_met,
            ),
        ];
        for (i, (label, value, color, check)) in tiles.iter().enumerate() {
            let (col, row) = ((i % 2) as f32, (i / 2) as f32);
            let rect = Rect::from_min_size(
                area.min + Vec2::new(col * (w + TILE_GAP), row * (TILE_H + TILE_GAP)),
                Vec2::new(w, TILE_H),
            );
            stat_tile(ui, rect, label, value, *color, *check);
        }
    });
}

fn draw_donut(ui: &egui::Ui, rect: Rect, stats: &DashStats) {
    let p = ui.painter_at(rect);
    let c = rect.center();
    let outer = (rect.height() * 0.5 - 2.0).min(rect.width() * 0.4);
    let thick = (outer * 0.30).clamp(10.0, 22.0);
    let radius = outer - thick * 0.5;

    let weights: Vec<f32> = if stats.task_hits.is_empty() {
        vec![35.0, 28.0, 22.0, 15.0]
    } else {
        stats.task_hits.iter().map(|h| *h as f32).collect()
    };
    let total: f32 = weights.iter().sum::<f32>().max(1.0);
    let gap = if weights.len() > 1 { 0.04 } else { 0.0 };
    let mut start = -FRAC_PI_2;
    for (i, w) in weights.iter().enumerate() {
        let sweep = w / total * TAU;
        let (a0, a1) = (start + gap * 0.5, start + sweep - gap * 0.5);
        let steps = ((a1 - a0).abs() * 18.0).ceil().max(2.0) as usize;
        let pts: Vec<Pos2> = (0..=steps)
            .map(|s| {
                let a = a0 + (a1 - a0) * s as f32 / steps as f32;
                c + Vec2::new(a.cos(), a.sin()) * radius
            })
            .collect();
        p.add(egui::Shape::line(pts, Stroke::new(thick, RING[i % 4])));
        start += sweep;
    }

    p.text(
        c - Vec2::new(0.0, 7.0),
        Align2::CENTER_CENTER,
        format!("{:.0}%", stats.accuracy_pct()),
        FontId::monospace(21.0),
        Color32::WHITE,
    );
    p.text(
        c + Vec2::new(0.0, 12.0),
        Align2::CENTER_CENTER,
        "Accuracy",
        FontId::proportional(11.0),
        widgets::TEXT_SUBTLE,
    );
}

/// Exact vertical accent-bar mini-stat card styling from 01-MAIN_REFERENCE.jpg:
/// Muted label on top, vertical colored accent bar immediately left of tabular value.
fn stat_tile(ui: &mut egui::Ui, rect: Rect, label: &str, value: &str, color: Color32, check: bool) {
    let p = ui.painter_at(rect);
    p.rect_filled(rect, egui::CornerRadius::same(8), widgets::SURFACE_2);
    p.rect_stroke(
        rect,
        egui::CornerRadius::same(8),
        Stroke::new(1.0, widgets::BORDER),
        egui::StrokeKind::Inside,
    );

    let x = rect.left() + 11.0;
    p.text(
        Pos2::new(x, rect.top() + 14.0),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(11.0),
        widgets::TEXT_SUBTLE,
    );

    // Vertical accent bar immediately to the left of the metric value
    let bar_rect = Rect::from_min_size(Pos2::new(x, rect.top() + 26.0), Vec2::new(2.5, 14.0));
    p.rect_filled(bar_rect, egui::CornerRadius::same(1), color);

    let value_x = x + 8.0;
    let value_y = rect.top() + 33.0;
    let galley = ui
        .painter()
        .layout_no_wrap(value.to_string(), FontId::monospace(14.5), Color32::WHITE);
    let width = galley.size().x;
    p.galley(
        Pos2::new(value_x, value_y - galley.size().y * 0.5),
        galley,
        Color32::WHITE,
    );

    if check {
        let c = Pos2::new(value_x + width + 12.0, value_y);
        let s = Stroke::new(1.6, widgets::POSITIVE);
        p.circle_stroke(c, 6.5, Stroke::new(1.2, widgets::POSITIVE));
        p.add(egui::Shape::line(
            vec![
                c + Vec2::new(-2.6, 0.0),
                c + Vec2::new(-0.6, 2.0),
                c + Vec2::new(2.8, -2.2),
            ],
            s,
        ));
    }
}
