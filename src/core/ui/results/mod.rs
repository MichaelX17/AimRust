pub mod accuracy_chart_card;
pub mod components;
pub mod layout;
pub mod score_chart_card;
pub mod session_card;
pub mod stats_card;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::EguiContexts;
use bevy_egui::egui::{self, Align, Layout, Pos2, Rect, Vec2};

use crate::core::round::RoundTimer;
use crate::core::score::{RoundScore, scoring_mode};
use crate::core::settings::UserSettings;
use crate::core::state::AppState;
use crate::core::storage::{LastOutcome, Scores};
use crate::core::task_api::{SelectedTask, TaskRegistry};
use crate::core::ui::widgets;

use accuracy_chart_card::render_accuracy_chart_card;
use layout::{ACTIONS_HEIGHT, SECTION_GAP, render_actions, render_twin_row};
use score_chart_card::render_score_chart_card;
use session_card::{SessionCardParams, render_session_card};
use stats_card::render_stats_card;

#[derive(SystemParam)]
pub struct ResultsData<'w> {
    score: Res<'w, RoundScore>,
    timer: Res<'w, RoundTimer>,
    registry: Res<'w, TaskRegistry>,
    selected: Res<'w, SelectedTask>,
    scores: Res<'w, Scores>,
    last: Res<'w, LastOutcome>,
    settings: Res<'w, UserSettings>,
}

pub fn results_ui(
    mut contexts: EguiContexts,
    windows: Query<&Window, With<PrimaryWindow>>,
    data: ResultsData,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };

    // Keyboard shortcuts: Space/Enter -> Play Again, Esc -> Go Menu
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        next_state.set(AppState::Menu);
        return;
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Space) || i.key_pressed(egui::Key::Enter)) {
        next_state.set(AppState::Countdown);
        return;
    }

    let size = crate::core::ui::page_size(ctx, window);
    let ResultsData {
        score,
        timer,
        registry,
        selected,
        scores,
        last,
        settings,
    } = data;

    let mode = scoring_mode(&registry, &selected);
    let duration = timer.duration.round().max(0.0) as u32;
    let duration_key = duration.to_string();
    let task_name = registry
        .0
        .iter()
        .find(|task| task.id == selected.0)
        .map(|task| task.name)
        .unwrap_or("Round");
    let task_scores = scores.tasks.get(selected.0);
    let highest_score = task_scores
        .and_then(|task| {
            let dur_best = task.best.get(&duration_key).map(|b| b.score);
            let all_best = task.best.values().map(|b| b.score).max();
            dur_best.or(all_best)
        })
        .unwrap_or(last.best);

    let doa_failed = settings.doa_mode_enabled && timer.doa_failed;

    widgets::page(ctx, "results_screen", size, |ui| {
        let inner = widgets::page_inner(size);
        let content_width = inner.x.min(1080.0);
        let content_x = (inner.x - content_width) * 0.5;

        let (outer_rect, _) = ui.allocate_exact_size(inner, egui::Sense::hover());
        let bounds = Rect::from_min_size(
            Pos2::new(outer_rect.left() + content_x, outer_rect.top()),
            Vec2::new(content_width, outer_rect.height()),
        );

        let mut content_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(bounds)
                .layout(Layout::top_down(Align::Min)),
        );
        content_ui.set_clip_rect(outer_rect);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(&mut content_ui, |ui| {
                ui.set_width(content_width);
                ui.spacing_mut().item_spacing.y = 0.0;

                let is_wide = content_width >= 680.0;
                let hero_h = 152.0;
                let chart_h = if is_wide { 216.0 } else { 196.0 };
                let resume_h = if content_width >= 560.0 { 150.0 } else { 226.0 };

                let total_h = if is_wide {
                    hero_h + chart_h + resume_h + ACTIONS_HEIGHT + SECTION_GAP * 4.0 + 12.0
                } else {
                    hero_h + chart_h * 2.0 + resume_h + ACTIONS_HEIGHT + SECTION_GAP * 5.0 + 12.0
                };
                let top_pad = ((inner.y - total_h) * 0.42).clamp(6.0, 40.0);
                ui.add_space(top_pad);

                // 1. Top Hero Card: Achieved Score & Highest Score
                render_session_card(
                    ui,
                    Vec2::new(content_width, hero_h),
                    SessionCardParams {
                        task_name,
                        duration,
                        mode,
                        score: &score,
                        timer: &timer,
                        highest_score,
                        new_best: last.new_best,
                        doa_failed,
                    },
                );
                ui.add_space(SECTION_GAP);

                // 2. Two Point Graphs: Last 5 Scores & Last 5 Accuracy %
                if is_wide {
                    render_twin_row(
                        ui,
                        content_width,
                        chart_h,
                        SECTION_GAP,
                        |ui, sz| render_score_chart_card(ui, sz, task_scores),
                        |ui, sz| render_accuracy_chart_card(ui, sz, task_scores),
                    );
                } else {
                    render_score_chart_card(ui, Vec2::new(content_width, chart_h), task_scores);
                    ui.add_space(SECTION_GAP);
                    render_accuracy_chart_card(ui, Vec2::new(content_width, chart_h), task_scores);
                }
                ui.add_space(SECTION_GAP);

                // 3. Stylized Resume: AVG Points (Last 5), AVG Accuracy (Last 5), Hits, Shots
                render_stats_card(
                    ui,
                    Vec2::new(content_width, resume_h),
                    &score,
                    task_scores,
                );
                ui.add_space(SECTION_GAP + 6.0);

                // 4. Stylized Go Menu & Play Again Action Bar
                render_actions(ui, content_width, &mut next_state);
                ui.add_space(16.0);
            });
    });
}
