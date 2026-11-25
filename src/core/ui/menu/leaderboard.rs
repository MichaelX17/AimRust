use bevy::prelude::*;
use bevy_egui::egui::{self, Align, Layout, Vec2};

use super::components::{self, CardIcon};
use crate::core::round::RoundDuration;
use crate::core::state::AppState;
use crate::core::storage::Scores;
use crate::core::task_api::{SelectedTask, TaskRegistry};
use crate::core::ui::widgets;

pub struct LeaderboardParams<'a> {
    pub registry: &'a TaskRegistry,
    pub selected: &'a mut SelectedTask,
    pub duration: &'a mut RoundDuration,
    pub scores: &'a Scores,
    pub search_query: &'a str,
    pub next_state: &'a mut NextState<AppState>,
}

pub fn render_leaderboard_view(ui: &mut egui::Ui, params: LeaderboardParams) {
    let LeaderboardParams {
        registry,
        selected,
        duration,
        scores,
        search_query,
        next_state,
    } = params;

    let size = Vec2::new(
        ui.available_width(),
        (ui.available_height() - 8.0).max(460.0),
    );
    let query = search_query.trim().to_ascii_lowercase();
    let mut entries: Vec<(&'static str, &'static str, i32, usize)> = registry
        .0
        .iter()
        .filter(|t| {
            query.is_empty()
                || t.name.to_ascii_lowercase().contains(&query)
                || t.id.to_ascii_lowercase().contains(&query)
        })
        .map(|t| {
            let ts = scores.tasks.get(t.id);
            let best = ts
                .and_then(|s| s.best.values().map(|b| b.score).max())
                .unwrap_or(0);
            let runs = ts.map(|s| s.recent.len()).unwrap_or(0);
            (t.id, t.name, best, runs)
        })
        .collect();
    entries.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.1.cmp(b.1)));

    widgets::fixed_card(ui, size, |ui| {
        components::card_header(
            ui,
            CardIcon::Bars,
            widgets::TARGET_YELLOW,
            "Personal Best Leaderboard",
            Some("Local High Scores across all scenarios"),
            |ui| {
                if components::cta_button(ui, "PLAY MULTISHOT ULTIMATE").clicked() {
                    selected.0 = "multishot_ultimate";
                    duration.0 = 60;
                    next_state.set(AppState::Countdown);
                }
            },
        );
        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .id_salt("dashboard_leaderboard_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (idx, (id, name, best, runs)) in entries.into_iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("#{:02}", idx + 1))
                                .monospace()
                                .size(13.0)
                                .strong()
                                .color(if idx == 0 {
                                    widgets::TARGET_YELLOW
                                } else {
                                    widgets::TEXT_WEAK
                                }),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(name)
                                .size(14.0)
                                .strong()
                                .color(widgets::TEXT),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if widgets::ghost_button(ui, "Play").clicked() {
                                selected.0 = id;
                                duration.0 = 60;
                                next_state.set(AppState::Countdown);
                            }
                            ui.add_space(12.0);
                            let score_text = if best > 0 {
                                widgets::thousands(best as i64)
                            } else {
                                "—".to_string()
                            };
                            ui.label(
                                egui::RichText::new(score_text)
                                    .monospace()
                                    .size(14.0)
                                    .strong()
                                    .color(widgets::TEXT),
                            );
                            ui.add_space(16.0);
                            ui.label(
                                egui::RichText::new(format!("{runs} runs"))
                                    .monospace()
                                    .size(12.0)
                                    .color(widgets::TEXT_SUBTLE),
                            );
                        });
                    });
                    ui.separator();
                }
            });
    });
}
