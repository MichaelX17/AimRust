use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::EguiContexts;
use bevy_egui::egui::{self, Align, Align2, FontId, Layout};

use crate::core::state::AppState;
use crate::core::storage::Scores;
use crate::core::task_api::{SelectedTask, TaskInfo, TaskRegistry};
use crate::core::ui::UiAccent;
use crate::core::ui::widgets;

const SIDEBAR_WIDTH: f32 = 200.0;
const SIDEBAR_NARROW: f32 = 168.0;
const SIDEBAR_GAP: f32 = 18.0;
const CARD_GAP: f32 = 12.0;
/// Below this content width the dashboard falls back to a single column.
const STACK_BREAKPOINT: f32 = 620.0;

#[derive(SystemParam)]
pub struct HomeData<'w, 's> {
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    registry: Res<'w, TaskRegistry>,
    selected: ResMut<'w, SelectedTask>,
    scores: Res<'w, Scores>,
    accent: Res<'w, UiAccent>,
}

pub fn menu_ui(
    mut contexts: EguiContexts,
    data: HomeData,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let HomeData {
        windows,
        registry,
        mut selected,
        scores,
        accent,
    } = data;
    let Ok(window) = windows.single() else {
        return;
    };
    let accent = accent.0;
    let size = crate::core::ui::page_size(ctx, window);

    widgets::page(ctx, "home_screen", size, |ui| {
        let inner = widgets::page_inner(size);
        let sidebar_width = if inner.x < 1050.0 {
            SIDEBAR_NARROW
        } else {
            SIDEBAR_WIDTH
        };

        // Place the sidebar and the dashboard with exact rects instead of
        // `horizontal_top` + item spacing: every pixel of the page width is
        // accounted for, so the dashboard can never drift right.
        let (page_rect, _) = ui.allocate_exact_size(inner, egui::Sense::hover());
        let sidebar_rect =
            egui::Rect::from_min_size(page_rect.min, egui::vec2(sidebar_width, page_rect.height()));
        let content_width = (page_rect.width() - sidebar_width - SIDEBAR_GAP).max(0.0);
        let content_rect = egui::Rect::from_min_size(
            egui::pos2(sidebar_rect.right() + SIDEBAR_GAP, page_rect.top()),
            egui::vec2(content_width, page_rect.height()),
        );

        let mut sidebar_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(sidebar_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        sidebar_ui.set_clip_rect(page_rect);
        sidebar(
            &mut sidebar_ui,
            accent,
            page_rect.height(),
            sidebar_width,
            &mut next_state,
            &mut exit,
        );

        let mut content_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        content_ui.set_clip_rect(page_rect);
        egui::ScrollArea::vertical()
            .max_width(content_width)
            .max_height(page_rect.height())
            .auto_shrink([false, false])
            .show(&mut content_ui, |ui| {
                dashboard(
                    ui,
                    &registry,
                    &mut selected,
                    &scores,
                    accent,
                    &mut next_state,
                );
            });
    });
}

fn sidebar(
    ui: &mut egui::Ui,
    accent: egui::Color32,
    height: f32,
    width: f32,
    next_state: &mut NextState<AppState>,
    exit: &mut MessageWriter<AppExit>,
) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    ui.painter().line_segment(
        [
            egui::pos2(rect.right(), rect.top()),
            egui::pos2(rect.right(), rect.bottom()),
        ],
        egui::Stroke::new(1.0, widgets::BORDER),
    );

    let mut ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(egui::vec2(12.0, 8.0)))
            .layout(Layout::top_down(Align::Min)),
    );

    ui.horizontal(|ui| {
        let (icon_rect, _) =
            ui.allocate_exact_size(egui::vec2(26.0, 26.0), egui::Sense::hover());
        let painter = ui.painter_at(icon_rect);
        painter.circle_filled(icon_rect.center(), 13.0, accent);
        painter.circle_stroke(
            icon_rect.center(),
            5.5,
            egui::Stroke::new(1.5, egui::Color32::WHITE),
        );
        painter.line_segment(
            [
                icon_rect.center() - egui::vec2(11.0, 0.0),
                icon_rect.center() - egui::vec2(6.0, 0.0),
            ],
            egui::Stroke::new(1.5, egui::Color32::WHITE),
        );
        painter.line_segment(
            [
                icon_rect.center() + egui::vec2(6.0, 0.0),
                icon_rect.center() + egui::vec2(11.0, 0.0),
            ],
            egui::Stroke::new(1.5, egui::Color32::WHITE),
        );
        ui.label(
            egui::RichText::new("AimRust")
                .size(18.0)
                .strong()
                .color(widgets::TEXT),
        );
    });
    ui.add_space(22.0);

    nav_item(&mut ui, "Home", Icon::Home, true, accent);
    if nav_item(&mut ui, "Play", Icon::Play, false, accent).clicked() {
        next_state.set(AppState::TaskSelect);
    }
    if nav_item(&mut ui, "Settings", Icon::Settings, false, accent).clicked() {
        next_state.set(AppState::Settings);
    }

    ui.add_space((ui.available_height() - 104.0).max(0.0));
    if nav_item(&mut ui, "Quit", Icon::Quit, false, accent).clicked() {
        exit.write(AppExit::Success);
    }
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        let (dot, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
        ui.painter().circle_filled(dot.center(), 4.0, widgets::POSITIVE);
        ui.label(
            egui::RichText::new("READY  v0.1.0")
                .size(11.0)
                .color(widgets::POSITIVE),
        );
    });
}

#[derive(Clone, Copy)]
enum Icon {
    Home,
    Play,
    Settings,
    Quit,
}

fn nav_item(
    ui: &mut egui::Ui,
    label: &str,
    icon: Icon,
    active: bool,
    accent: egui::Color32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 38.0),
        egui::Sense::click(),
    );
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let hovered = response.hovered();

    if active || hovered {
        let fill = if active {
            accent
        } else {
            widgets::SURFACE_2
        };
        ui.painter().rect_filled(rect, egui::CornerRadius::same(10), fill);
    }

    let color = if active {
        egui::Color32::WHITE
    } else {
        widgets::TEXT_WEAK
    };
    draw_icon(ui, icon, rect, color);
    ui.painter().text(
        egui::pos2(rect.left() + 40.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(15.0),
        color,
    );
    response
}

fn draw_icon(ui: &egui::Ui, icon: Icon, rect: egui::Rect, color: egui::Color32) {
    let painter = ui.painter_at(rect);
    let stroke = egui::Stroke::new(1.6, color);
    let c = egui::pos2(rect.left() + 20.0, rect.center().y);

    match icon {
        Icon::Home => {
            painter.line_segment(
                [
                    egui::pos2(c.x - 7.0, c.y),
                    egui::pos2(c.x, c.y - 7.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(c.x, c.y - 7.0),
                    egui::pos2(c.x + 7.0, c.y),
                ],
                stroke,
            );
            painter.rect_stroke(
                egui::Rect::from_min_max(
                    egui::pos2(c.x - 5.0, c.y),
                    egui::pos2(c.x + 5.0, c.y + 7.0),
                ),
                egui::CornerRadius::ZERO,
                stroke,
                egui::StrokeKind::Inside,
            );
        }
        Icon::Play => {
            painter.add(egui::Shape::convex_polygon(
                vec![
                    egui::pos2(c.x - 4.0, c.y - 6.0),
                    egui::pos2(c.x + 6.0, c.y),
                    egui::pos2(c.x - 4.0, c.y + 6.0),
                ],
                color,
                egui::Stroke::NONE,
            ));
        }
        Icon::Settings => {
            for offset in [-5.0, 0.0, 5.0] {
                painter.line_segment(
                    [
                        egui::pos2(c.x - 8.0, c.y + offset),
                        egui::pos2(c.x + 8.0, c.y + offset),
                    ],
                    stroke,
                );
            }
            painter.circle_filled(egui::pos2(c.x - 3.0, c.y - 5.0), 2.2, color);
            painter.circle_filled(egui::pos2(c.x + 3.0, c.y), 2.2, color);
            painter.circle_filled(egui::pos2(c.x - 2.0, c.y + 5.0), 2.2, color);
        }
        Icon::Quit => {
            painter.line_segment(
                [
                    egui::pos2(c.x - 7.0, c.y - 7.0),
                    egui::pos2(c.x - 7.0, c.y + 7.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(c.x - 7.0, c.y - 7.0),
                    egui::pos2(c.x + 2.0, c.y - 7.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(c.x - 7.0, c.y + 7.0),
                    egui::pos2(c.x + 2.0, c.y + 7.0),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(c.x - 2.0, c.y),
                    egui::pos2(c.x + 7.0, c.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(c.x + 3.0, c.y - 4.0),
                    egui::pos2(c.x + 7.0, c.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(c.x + 3.0, c.y + 4.0),
                    egui::pos2(c.x + 7.0, c.y),
                ],
                stroke,
            );
        }
    }
}

fn dashboard(
    ui: &mut egui::Ui,
    registry: &TaskRegistry,
    selected: &mut SelectedTask,
    scores: &Scores,
    accent: egui::Color32,
    next_state: &mut NextState<AppState>,
) {
    ui.horizontal(|ui| {
        widgets::small_label(ui, "/AIMRUST / HOME");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            widgets::chip(ui, "TELEMETRY ENGINE · READY", widgets::POSITIVE);
        });
    });
    ui.add_space(12.0);

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new("Welcome back, Player")
                    .size(34.0)
                    .strong()
                    .color(widgets::TEXT),
            );
            widgets::muted(ui, "Ready to improve your aim? Your drills are set.");
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::primary_button(ui, "QUICK PLAY  >", accent).clicked() {
                next_state.set(AppState::TaskSelect);
            }
        });
    });
    ui.add_space(16.0);

    // Every width and height in the card grid comes from this one measurement.
    let total_width = ui.available_width();
    let available_height = ui.available_height();
    if total_width < 32.0 || available_height < 32.0 {
        return;
    }

    // Two equal-height rows that always fit the space below the header. No
    // minimum on `bottom_height`: if the window is very short the cards get
    // shorter (content is clipped by `fixed_card`) instead of pushing the rows
    // out of alignment.
    let top_height = (available_height * 0.31)
        .clamp(185.0, 240.0)
        .min(available_height);
    let bottom_height = (available_height - top_height - CARD_GAP).max(0.0);

    if total_width < STACK_BREAKPOINT {
        let cards_height = top_height * 3.0 + bottom_height * 2.0 + CARD_GAP * 4.0;
        let (cards_rect, _) =
            ui.allocate_exact_size(egui::vec2(total_width, cards_height), egui::Sense::hover());
        let mut y = cards_rect.top();
        let mut next = |height: f32| {
            let rect = egui::Rect::from_min_size(
                egui::pos2(cards_rect.left(), y),
                egui::vec2(cards_rect.width(), height),
            );
            y += height + CARD_GAP;
            rect
        };

        let rect = next(top_height);
        place_card(ui, rect, |ui| {
            personal_best_card(ui, rect.width(), rect.height(), registry, scores);
        });
        let rect = next(top_height);
        place_card(ui, rect, |ui| {
            recent_card(ui, rect.width(), rect.height(), scores, accent);
        });
        let rect = next(top_height);
        place_card(ui, rect, |ui| {
            overall_card(ui, rect.width(), rect.height(), scores);
        });
        let rect = next(bottom_height);
        place_card(ui, rect, |ui| {
            tasks_card(
                ui,
                rect.width(),
                rect.height(),
                registry,
                selected,
                scores,
                accent,
                next_state,
            );
        });
        let rect = next(bottom_height);
        place_card(ui, rect, |ui| {
            session_card(ui, rect.width(), rect.height(), scores);
        });
        return;
    }

    // One column system for the whole grid:
    //   left_width = first + gap + second = task selection width
    //   right_width = overall = session width
    //   left_width + gap + right_width = total_width
    let left_width = (total_width - CARD_GAP) * 0.64;
    let right_width = total_width - CARD_GAP - left_width;
    let pair_width = (left_width - CARD_GAP).max(0.0);
    let first_width = pair_width * (30.0 / 63.0);
    let second_width = pair_width - first_width;

    let cards_height = top_height + CARD_GAP + bottom_height;
    let (cards_rect, _) =
        ui.allocate_exact_size(egui::vec2(total_width, cards_height), egui::Sense::hover());

    let left = cards_rect.left();
    let top = cards_rect.top();
    let bottom = top + top_height + CARD_GAP;
    let right_left = left + left_width + CARD_GAP;

    let personal_best_rect =
        egui::Rect::from_min_size(egui::pos2(left, top), egui::vec2(first_width, top_height));
    let recent_rect = egui::Rect::from_min_size(
        egui::pos2(left + first_width + CARD_GAP, top),
        egui::vec2(second_width, top_height),
    );
    let overall_rect = egui::Rect::from_min_size(
        egui::pos2(right_left, top),
        egui::vec2(right_width, top_height),
    );
    let tasks_rect = egui::Rect::from_min_size(
        egui::pos2(left, bottom),
        egui::vec2(left_width, bottom_height),
    );
    let session_rect = egui::Rect::from_min_size(
        egui::pos2(right_left, bottom),
        egui::vec2(right_width, bottom_height),
    );

    place_card(ui, personal_best_rect, |ui| {
        personal_best_card(
            ui,
            personal_best_rect.width(),
            personal_best_rect.height(),
            registry,
            scores,
        );
    });
    place_card(ui, recent_rect, |ui| {
        recent_card(
            ui,
            recent_rect.width(),
            recent_rect.height(),
            scores,
            accent,
        );
    });
    place_card(ui, overall_rect, |ui| {
        overall_card(ui, overall_rect.width(), overall_rect.height(), scores);
    });
    place_card(ui, tasks_rect, |ui| {
        tasks_card(
            ui,
            tasks_rect.width(),
            tasks_rect.height(),
            registry,
            selected,
            scores,
            accent,
            next_state,
        );
    });
    place_card(ui, session_rect, |ui| {
        session_card(ui, session_rect.width(), session_rect.height(), scores);
    });
}

/// Runs `add` inside a child UI whose bounds are exactly `rect`, so the card
/// painted by `fixed_card` cannot grow or shift the surrounding grid.
fn place_card(ui: &mut egui::Ui, rect: egui::Rect, add: impl FnOnce(&mut egui::Ui)) {
    let clip = ui.clip_rect().intersect(rect);
    let mut card_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::top_down(Align::Min)),
    );
    card_ui.set_clip_rect(clip);
    add(&mut card_ui);
}

fn best_task(scores: &Scores, registry: &TaskRegistry) -> Option<(i32, &'static str, u32)> {
    let mut best: Option<(i32, &'static str, u32)> = None;
    for (task_id, task_scores) in &scores.tasks {
        for (duration, record) in &task_scores.best {
            if best.is_none_or(|(score, _, _)| record.score > score) {
                let name = registry
                    .0
                    .iter()
                    .find(|task| task.id == task_id.as_str())
                    .map(|task| task.name)
                    .unwrap_or("Unknown");
                best = Some((record.score, name, duration.parse().unwrap_or(0)));
            }
        }
    }
    best
}

fn recent_records(scores: &Scores) -> Vec<(u64, i32)> {
    let mut records: Vec<(u64, i32)> = scores
        .tasks
        .values()
        .flat_map(|task| task.recent.iter().map(|round| (round.unix_secs, round.score)))
        .collect();
    records.sort_by_key(|(timestamp, _)| *timestamp);
    records
}

fn totals(scores: &Scores) -> (u32, u32, u32) {
    let mut hits = 0u32;
    let mut misses = 0u32;
    let mut rounds = 0u32;
    for task in scores.tasks.values() {
        for round in &task.recent {
            hits += round.hits;
            misses += round.misses;
            rounds += 1;
        }
    }
    (hits, misses, rounds)
}

fn personal_best_card(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    registry: &TaskRegistry,
    scores: &Scores,
) {
    widgets::fixed_card(ui, egui::vec2(width, height), |ui| {
        ui.horizontal(|ui| {
            widgets::small_label(ui, "Personal best");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(34.0, 34.0), egui::Sense::hover());
                draw_trophy(ui, rect);
            });
        });
        ui.add_space(4.0);

        match best_task(scores, registry) {
            Some((score, name, duration)) => {
                widgets::big_value(ui, widgets::thousands(score as i64), 38.0);
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(name)
                            .size(14.0)
                            .strong()
                            .color(widgets::TEXT),
                    );
                    widgets::chip(ui, "BEST", widgets::POSITIVE);
                });
                widgets::muted(ui, format!("{duration} s round"));
            }
            None => {
                widgets::big_value(ui, "—", 38.0);
                widgets::muted(ui, "No rounds yet — press Play to start.");
            }
        }
    });
}

fn draw_trophy(ui: &egui::Ui, rect: egui::Rect) {
    let painter = ui.painter_at(rect);
    let color = widgets::TEXT_WEAK;
    let stroke = egui::Stroke::new(1.4, color);
    let center = rect.center();
    painter.rect_stroke(
        egui::Rect::from_center_size(
            egui::pos2(center.x, rect.top() + 12.0),
            egui::vec2(18.0, 14.0),
        ),
        egui::CornerRadius::same(4),
        stroke,
        egui::StrokeKind::Inside,
    );
    painter.line_segment(
        [
            egui::pos2(center.x, rect.top() + 19.0),
            egui::pos2(center.x, rect.top() + 26.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            egui::pos2(center.x - 7.0, rect.top() + 27.0),
            egui::pos2(center.x + 7.0, rect.top() + 27.0),
        ],
        stroke,
    );
}

fn recent_card(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    scores: &Scores,
    accent: egui::Color32,
) {
    widgets::fixed_card(ui, egui::vec2(width, height), |ui| {
        ui.horizontal(|ui| {
            widgets::small_label(ui, "Recent rounds");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                widgets::chip(ui, "LAST 15", widgets::TEXT_WEAK);
            });
        });
        ui.add_space(6.0);

        let records = recent_records(scores);
        let values: Vec<f32> = records
            .iter()
            .rev()
            .take(15)
            .rev()
            .map(|(_, score)| (*score).max(0) as f32)
            .collect();
        widgets::bar_chart(ui, &values, accent, (height * 0.40).clamp(48.0, 110.0));

        ui.add_space(6.0);
        let peak = values.iter().copied().fold(0.0f32, f32::max);
        let average = if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f32>() / values.len() as f32
        };
        ui.horizontal(|ui| {
            widgets::muted(ui, format!("AVG: {}", widgets::thousands(average as i64)));
            ui.add_space(14.0);
            widgets::muted(ui, format!("PEAK: {}", widgets::thousands(peak as i64)));
        });
    });
}

fn progress_bar(ui: &mut egui::Ui, fraction: f32, color: egui::Color32) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 5.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(3), widgets::SURFACE_2);
    let filled = egui::Rect::from_min_size(
        rect.min,
        egui::vec2(rect.width() * fraction.clamp(0.0, 1.0), rect.height()),
    );
    ui.painter()
        .rect_filled(filled, egui::CornerRadius::same(3), color);
}

fn accuracy(scores: &Scores) -> f32 {
    let (hits, misses, _) = totals(scores);
    if hits + misses == 0 {
        0.0
    } else {
        hits as f32 / (hits + misses) as f32 * 100.0
    }
}

fn overall_card(ui: &mut egui::Ui, width: f32, height: f32, scores: &Scores) {
    widgets::fixed_card(ui, egui::vec2(width, height), |ui| {
        ui.horizontal(|ui| {
            widgets::small_label(ui, "Stats overall");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                widgets::chip(ui, "LAST 15", widgets::TEXT_WEAK);
            });
        });

        let accuracy = accuracy(scores);
        ui.horizontal(|ui| {
            widgets::big_value(ui, format!("{accuracy:.0}%"), 32.0);
            widgets::small_label(ui, "Accuracy");
        });
        ui.add_space(2.0);
        progress_bar(ui, accuracy / 100.0, widgets::POSITIVE);
        ui.add_space(4.0);

        let (hits, misses, rounds) = totals(scores);
        egui::Grid::new("overall_stats")
            .num_columns(3)
            .spacing([12.0, 2.0])
            .show(ui, |ui| {
                widgets::small_label(ui, "Hits");
                widgets::small_label(ui, "Misses");
                widgets::small_label(ui, "Rounds");
                ui.end_row();

                ui.label(
                    egui::RichText::new(hits.to_string())
                        .size(16.0)
                        .strong()
                        .color(widgets::POSITIVE),
                );
                ui.label(
                    egui::RichText::new(misses.to_string())
                        .size(16.0)
                        .strong()
                        .color(widgets::NEGATIVE),
                );
                ui.label(
                    egui::RichText::new(rounds.to_string())
                        .size(16.0)
                        .strong()
                        .color(widgets::TEXT),
                );
                ui.end_row();
            });
    });
}

fn task_best(scores: &Scores, task_id: &str) -> Option<i32> {
    scores
        .tasks
        .get(task_id)
        .and_then(|task| task.best.values().map(|best| best.score).max())
}

#[allow(clippy::too_many_arguments)]
fn tasks_card(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    registry: &TaskRegistry,
    selected: &mut SelectedTask,
    scores: &Scores,
    accent: egui::Color32,
    next_state: &mut NextState<AppState>,
) {
    widgets::fixed_card(ui, egui::vec2(width, height), |ui| {
        ui.horizontal(|ui| {
            widgets::small_label(ui, "Task selection");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::ghost_button(ui, "VIEW ALL  >").clicked() {
                    next_state.set(AppState::TaskSelect);
                }
            });
        });
        ui.add_space(8.0);

        if registry.0.is_empty() {
            widgets::muted(ui, "No tasks installed");
            return;
        }

        let mut chosen: Option<&'static str> = None;
        egui::ScrollArea::vertical()
            .id_salt("home_task_rows")
            .max_height((height - 86.0).max(80.0))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for task in &registry.0 {
                    if task_row(ui, task, task_best(scores, task.id), accent) {
                        chosen = Some(task.id);
                    }
                    ui.add_space(8.0);
                }
            });

        if let Some(id) = chosen {
            selected.0 = id;
            next_state.set(AppState::TimeSelect);
        }
    });
}

fn task_row(
    ui: &mut egui::Ui,
    task: &TaskInfo,
    best: Option<i32>,
    accent: egui::Color32,
) -> bool {
    let mut clicked = false;
    widgets::tile().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(task.name)
                        .size(15.0)
                        .strong()
                        .color(widgets::TEXT),
                );
                ui.label(
                    egui::RichText::new(task.description)
                        .size(11.0)
                        .color(widgets::TEXT_WEAK),
                );
                ui.add_space(4.0);
                match best {
                    Some(score) => {
                        widgets::chip(
                            ui,
                            &format!("BEST {}", widgets::thousands(score as i64)),
                            widgets::POSITIVE,
                        );
                    }
                    None => {
                        widgets::chip(ui, "UNPLAYED", widgets::TEXT_WEAK);
                    }
                }
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::primary_button(ui, "PLAY", accent).clicked() {
                    clicked = true;
                }
            });
        });
    });
    clicked
}

fn session_card(ui: &mut egui::Ui, width: f32, height: f32, scores: &Scores) {
    widgets::fixed_card(ui, egui::vec2(width, height), |ui| {
        widgets::small_label(ui, "Session stats");
        ui.add_space(6.0);

        let accuracy = accuracy(scores);
        widgets::big_value(ui, format!("{accuracy:.0}%"), 34.0);
        widgets::small_label(ui, "Accuracy");
        ui.add_space(6.0);
        progress_bar(ui, accuracy / 100.0, widgets::POSITIVE);
        ui.add_space(14.0);

        let (hits, misses, rounds) = totals(scores);
        stat_line(ui, "Rounds", rounds.to_string(), widgets::TEXT);
        stat_line(ui, "Hits", hits.to_string(), widgets::POSITIVE);
        stat_line(ui, "Misses", misses.to_string(), widgets::NEGATIVE);
    });
}

fn stat_line(ui: &mut egui::Ui, label: &str, value: String, color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .size(12.0)
                .color(widgets::TEXT_WEAK),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(egui::RichText::new(value).size(14.0).strong().color(color));
        });
    });
    ui.add_space(6.0);
}
