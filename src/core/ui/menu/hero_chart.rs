use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, Pos2, Sense, Vec2};

use super::chart;
use super::components::{self, CardIcon};
use super::stats::{DashStats, date_label};
use crate::core::round::RoundDuration;
use crate::core::state::AppState;
use crate::core::storage::Scores;
use crate::core::task_api::SelectedTask;
use crate::core::ui::widgets;

const HERO_TASK_ID: &str = "multishot_ultimate";
const CURVE_STROKE: Color32 = Color32::from_rgb(232, 246, 255);

pub struct HeroChartParams<'a> {
    pub selected: &'a mut SelectedTask,
    pub duration: &'a mut RoundDuration,
    pub scores: &'a Scores,
    pub stats: &'a DashStats,
    pub next_state: &'a mut NextState<AppState>,
}

pub fn render_hero_chart(ui: &mut egui::Ui, size: Vec2, params: HeroChartParams) {
    let HeroChartParams {
        selected,
        duration,
        scores,
        stats,
        next_state,
    } = params;

    let button_label = if size.x < 560.0 {
        "PLAY MULTISHOT"
    } else {
        "PLAY MULTISHOT ULTIMATE"
    };

    // Prefer Multishot Ultimate's last 28 sessions when available; otherwise show
    // overall session progression (or clean sample baseline on a fresh install).
    let task_scores = scores.tasks.get(HERO_TASK_ID);
    let task_series: Vec<(u64, f32)> = match task_scores {
        Some(ts) => {
            let count = ts.recent.len().min(28);
            ts.recent[ts.recent.len() - count..]
                .iter()
                .map(|r| (r.unix_secs, r.score.max(0) as f32))
                .collect()
        }
        None => Vec::new(),
    };

    let series_points: &[(u64, f32)] = if task_series.len() >= 2 {
        &task_series
    } else {
        &stats.series
    };

    let headline_score = task_scores
        .and_then(|ts| {
            let from_best = ts.best.values().map(|b| b.score).max();
            let from_recent = ts.recent.iter().map(|r| r.score).max();
            from_best.max(from_recent)
        })
        .unwrap_or_else(|| {
            series_points
                .last()
                .map(|(_, v)| v.round() as i32)
                .unwrap_or(0)
        });

    let subtitle = if stats.sample {
        "Aim Rating · Last 28 Sessions (Sample)"
    } else {
        "Aim Rating · Last 28 Sessions"
    };

    widgets::fixed_card(ui, size, |ui| {
        components::card_header(
            ui,
            CardIcon::Trend,
            widgets::ACCENT_CYAN,
            "Score Progression",
            Some(subtitle),
            |ui| {
                if components::cta_button(ui, button_label).clicked() {
                    selected.0 = HERO_TASK_ID;
                    duration.0 = 60;
                    next_state.set(AppState::Countdown);
                }
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.label(
                        egui::RichText::new("AIM RATING")
                            .size(10.0)
                            .strong()
                            .color(widgets::TEXT_WEAK),
                    );
                    ui.label(
                        egui::RichText::new(widgets::thousands(headline_score as i64))
                            .monospace()
                            .size(18.0)
                            .strong()
                            .color(widgets::TEXT),
                    );
                });
            },
        );
        ui.add_space(6.0);

        let area = Vec2::new(ui.available_width(), ui.available_height().max(60.0));
        let (rect, response) = ui.allocate_exact_size(area, Sense::hover());
        draw_chart(ui, rect, &response, series_points);
    });
}

fn draw_chart(ui: &egui::Ui, rect: egui::Rect, response: &egui::Response, series: &[(u64, f32)]) {
    let painter = ui.painter_at(rect);
    let n = series.len();
    if n == 0 {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Finish a run to chart your Score Progression",
            egui::FontId::proportional(12.0),
            widgets::TEXT_WEAK,
        );
        return;
    }

    let plot = chart::plot_rect(rect, 44.0);
    let peak = series.iter().map(|s| s.1).fold(0.0, f32::max);
    let max = chart::nice_max(peak * 1.12);

    let tick_count = 5.min(n).max(1);
    let x_labels: Vec<String> = (0..tick_count)
        .map(|k| {
            let idx = if tick_count == 1 {
                0
            } else {
                k * (n - 1) / (tick_count - 1)
            };
            let time = series[idx].0;
            if time > 0 {
                date_label(time)
            } else {
                format!("#{}", idx + 1)
            }
        })
        .collect();
    chart::grid(
        &painter,
        plot,
        &chart::y_scale(max, 5, chart::axis_value),
        &x_labels,
        false,
    );

    let pts: Vec<Pos2> = series
        .iter()
        .enumerate()
        .map(|(i, (_, v))| {
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

        // Smooth white/cyan vertical gradient area fill matching 01-MAIN_REFERENCE.jpg
        chart::area_fill(
            &painter,
            &curve,
            plot.bottom(),
            Color32::from_rgba_unmultiplied(210, 238, 255, 76),
            Color32::from_rgba_unmultiplied(56, 188, 248, 0),
        );
        chart::glow_line(&painter, &curve, CURVE_STROKE, 2.2);
    }

    let focus = if n == 1 {
        0
    } else {
        match response.hover_pos() {
            Some(m) if plot.expand2(Vec2::new(0.0, 40.0)).contains(m) => {
                let f = ((m.x - plot.left()) / plot.width()).clamp(0.0, 1.0);
                (f * (n - 1) as f32).round() as usize
            }
            _ => (n / 2).min(n - 1),
        }
    };
    let value = series[focus].1;
    let first = series[0].1;
    let delta = (n >= 2 && first > 0.0).then(|| {
        let pct = (value / first - 1.0) * 100.0;
        (
            format!("{} {:.1}%", if pct >= 0.0 { "↑" } else { "↓" }, pct.abs()),
            pct >= 0.0,
        )
    });
    chart::node_with_tooltip(
        &painter,
        pts[focus],
        rect,
        &widgets::thousands(value as i64),
        delta,
    );
}
