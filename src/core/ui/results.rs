use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::{EguiContexts, egui};

use crate::core::hud::format_time;
use crate::core::round::RoundTimer;
use crate::core::score::{RoundScore, click_accuracy, scoring_mode};
use crate::core::state::AppState;
use crate::core::storage::{LastOutcome, Scores};
use crate::core::task_api::{ScoringMode, SelectedTask, TaskRegistry};
use crate::core::ui::UiAccent;
use crate::core::ui::widgets;

#[derive(SystemParam)]
pub struct ResultsData<'w> {
    score: Res<'w, RoundScore>,
    timer: Res<'w, RoundTimer>,
    registry: Res<'w, TaskRegistry>,
    selected: Res<'w, SelectedTask>,
    scores: Res<'w, Scores>,
    last: Res<'w, LastOutcome>,
}

pub fn results_ui(
    mut contexts: EguiContexts,
    windows: Query<&Window, With<PrimaryWindow>>,
    data: ResultsData,
    accent: Res<UiAccent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let size = crate::core::ui::page_size(ctx, window);
    let ResultsData {
        score,
        timer,
        registry,
        selected,
        scores,
        last,
    } = data;

    let accent = accent.0;
    let mode = scoring_mode(&registry, &selected);
    let duration = timer.duration.round().max(0.0) as u32;
    let duration_key = duration.to_string();
    let task_name = registry
        .0
        .iter()
        .find(|task| task.id == selected.0)
        .map(|task| task.name)
        .unwrap_or("Round");
    let best = scores
        .tasks
        .get(selected.0)
        .and_then(|task| task.best.get(&duration_key))
        .map(|best| best.score);

    let inner = widgets::page_inner(size);
    let card_width = (inner.x - 32.0).clamp(300.0, 460.0);
    let top = ((inner.y - 560.0) * 0.5).max(8.0);

    widgets::page(ctx, "results_screen", size, |ui| {
        egui::ScrollArea::vertical()
            .max_height(inner.y)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(top);
                    ui.allocate_ui_with_layout(
                        egui::vec2(card_width, 0.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            widgets::card().show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                header(ui, accent, task_name, duration, last.new_best);
                                ui.add_space(10.0);
                                headline(ui, mode, &score, &timer);
                                ui.add_space(14.0);
                                stat_tiles(ui, mode, &score, duration);
                                ui.add_space(10.0);
                                best_row(ui, duration, best);
                                ui.add_space(12.0);
                                recent_list(ui, scores.tasks.get(selected.0));
                                ui.add_space(16.0);
                                buttons(ui, accent, &mut next_state);
                            });
                        },
                    );
                });
            });
    });
}

fn header(
    ui: &mut egui::Ui,
    accent: egui::Color32,
    task_name: &str,
    duration: u32,
    new_best: bool,
) {
    ui.horizontal(|ui| {
        widgets::chip(ui, "ROUND COMPLETE", accent);
        if new_best {
            widgets::chip(ui, "NEW BEST", widgets::POSITIVE);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            widgets::muted(ui, format!("{task_name} · {}", format_time(duration)));
        });
    });
}

fn headline(
    ui: &mut egui::Ui,
    mode: ScoringMode,
    score: &RoundScore,
    timer: &RoundTimer,
) {
    match mode {
        ScoringMode::Click => {
            widgets::big_value(ui, widgets::thousands(score.points.max(0) as i64), 48.0);
            widgets::small_label(ui, "Score");
        }
        ScoringMode::Tracking => {
            widgets::big_value(
                ui,
                format!("{} ms", score.tracking_ms.floor() as u64),
                40.0,
            );
            let round_ms = timer.duration as f64 * 1000.0;
            let percent = if round_ms > 0.0 {
                score.tracking_ms / round_ms * 100.0
            } else {
                0.0
            };
            widgets::small_label(ui, format!("On target · {percent:.1}% of round"));
        }
    }
}

fn tile(ui: &mut egui::Ui, label: &str, value: String, color: egui::Color32) {
    widgets::tile().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            egui::RichText::new(label)
                .size(10.0)
                .strong()
                .color(widgets::TEXT_WEAK),
        );
        ui.label(egui::RichText::new(value).size(18.0).strong().color(color));
    });
}

fn stat_tiles(ui: &mut egui::Ui, mode: ScoringMode, score: &RoundScore, duration: u32) {
    let width = (ui.available_width() - 16.0) / 3.0;
    ui.horizontal(|ui| match mode {
        ScoringMode::Click => {
            ui.allocate_ui_with_layout(
                egui::vec2(width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| tile(ui, "HITS", score.hits.to_string(), widgets::POSITIVE),
            );
            ui.allocate_ui_with_layout(
                egui::vec2(width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| tile(ui, "MISSES", score.misses.to_string(), widgets::NEGATIVE),
            );
            ui.allocate_ui_with_layout(
                egui::vec2(width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    tile(
                        ui,
                        "ACCURACY",
                        format!("{:.1}%", click_accuracy(score)),
                        widgets::TEXT,
                    );
                },
            );
        }
        ScoringMode::Tracking => {
            ui.allocate_ui_with_layout(
                egui::vec2(width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    tile(
                        ui,
                        "TIME ON TARGET",
                        format!("{} ms", score.tracking_ms.floor() as u64),
                        widgets::POSITIVE,
                    );
                },
            );
            ui.allocate_ui_with_layout(
                egui::vec2(width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| tile(ui, "ROUND", format!("{duration} s"), widgets::TEXT),
            );
            ui.allocate_ui_with_layout(
                egui::vec2(width, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| tile(ui, "HITS", score.hits.to_string(), widgets::TEXT),
            );
        }
    });
}

fn best_row(ui: &mut egui::Ui, duration: u32, best: Option<i32>) {
    widgets::tile().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("BEST · {}", format_time(duration)))
                    .size(11.0)
                    .strong()
                    .color(widgets::TEXT_WEAK),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                match best {
                    Some(best) => {
                        ui.label(
                            egui::RichText::new(widgets::thousands(best as i64))
                                .size(18.0)
                                .strong()
                                .color(widgets::TEXT),
                        );
                    }
                    None => {
                        ui.label(egui::RichText::new("—").color(widgets::TEXT_WEAK));
                    }
                }
            });
        });
    });
}

fn recent_list(ui: &mut egui::Ui, task: Option<&crate::core::storage::TaskScores>) {
    widgets::small_label(ui, "Recent rounds");
    ui.add_space(4.0);
    egui::ScrollArea::vertical()
        .max_height(130.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let Some(task) = task else {
                widgets::muted(ui, "No history yet");
                return;
            };
            for record in task.recent.iter().rev().take(15) {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(widgets::thousands(record.score as i64))
                            .size(13.0)
                            .strong()
                            .color(widgets::TEXT),
                    );
                    widgets::muted(
                        ui,
                        format!(
                            "{}/{} hits   ·   {} s",
                            record.hits, record.misses, record.duration
                        ),
                    );
                });
            }
        });
}

fn buttons(ui: &mut egui::Ui, accent: egui::Color32, next_state: &mut NextState<AppState>) {
    ui.horizontal(|ui| {
        if widgets::primary_button(ui, "REPEAT", accent).clicked() {
            next_state.set(AppState::Countdown);
        }
        if widgets::ghost_button(ui, "BACK TO MENU").clicked() {
            next_state.set(AppState::Menu);
        }
    });
}
