use bevy::prelude::*;
use bevy_egui::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};

use super::components::{self, CardIcon};
use crate::core::round::RoundDuration;
use crate::core::state::AppState;
use crate::core::storage::Scores;
use crate::core::task_api::{SelectedTask, TaskRegistry};
use crate::core::ui::widgets;

/// Short subtitles for the featured modes; other tasks fall back to their description.
const FEATURED: [(&str, &str); 4] = [
    ("multishot_ultimate", "Flicking · Dynamic Targets"),
    ("gridshot", "Speed · Large Targets"),
    ("micro_adjust", "Precision · Micro Adjustments"),
    ("cylinder_micro", "Precision · High Verticality"),
];
const ROW_H: f32 = 50.0;

pub struct TaskRosterParams<'a> {
    pub registry: &'a TaskRegistry,
    pub selected: &'a mut SelectedTask,
    pub duration: &'a mut RoundDuration,
    pub scores: &'a Scores,
    pub search_query: &'a str,
    pub next_state: &'a mut NextState<AppState>,
}

pub fn render_task_roster(ui: &mut egui::Ui, size: Vec2, params: TaskRosterParams) {
    let TaskRosterParams {
        registry,
        selected,
        duration,
        scores,
        search_query,
        next_state,
    } = params;
    widgets::fixed_card(ui, size, |ui| {
        components::card_header(
            ui,
            CardIcon::Clipboard,
            widgets::TARGET_YELLOW,
            "Your Tasks",
            None,
            |ui| {
                let link = ui.add(
                    egui::Button::new(
                        egui::RichText::new("View all")
                            .size(12.0)
                            .color(widgets::TEXT_WEAK),
                    )
                    .frame(false),
                );
                if link.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    next_state.set(AppState::TaskSelect);
                }
            },
        );
        ui.add_space(6.0);
        column_header(ui);

        let query = search_query.trim().to_ascii_lowercase();

        // Featured modes first (in order), then any other registered task.
        let mut rows: Vec<(&'static str, &'static str, String)> = Vec::new();
        for (id, sub) in FEATURED {
            if let Some(t) = registry.0.iter().find(|t| t.id == id) {
                rows.push((t.id, t.name, sub.to_string()));
            }
        }
        for t in &registry.0 {
            if !FEATURED.iter().any(|(id, _)| *id == t.id) {
                rows.push((t.id, t.name, t.description.to_string()));
            }
        }
        if !query.is_empty() {
            rows.retain(|(id, name, sub)| {
                name.to_ascii_lowercase().contains(&query)
                    || sub.to_ascii_lowercase().contains(&query)
                    || id.to_ascii_lowercase().contains(&query)
            });
        }

        let mut clicked: Option<&'static str> = None;
        egui::ScrollArea::vertical()
            .id_salt("dashboard_task_roster_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                if rows.is_empty() {
                    ui.add_space(18.0);
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            egui::RichText::new("No matching tasks")
                                .size(12.5)
                                .color(widgets::TEXT_WEAK),
                        );
                    });
                }
                for (id, name, sub) in &rows {
                    let best = scores
                        .tasks
                        .get(*id)
                        .and_then(|t| t.best.values().map(|b| b.score).max());
                    if task_row(ui, name, sub, best, selected.0 == *id) {
                        clicked = Some(*id);
                    }
                }
            });

        if let Some(id) = clicked {
            selected.0 = id;
            duration.0 = 60;
            next_state.set(AppState::Countdown);
        }
    });
}

fn column_header(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 26.0), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, egui::CornerRadius::same(6), widgets::SURFACE_2);
    let font = FontId::proportional(11.0);
    p.text(
        Pos2::new(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        "Task",
        font.clone(),
        widgets::TEXT_SUBTLE,
    );
    p.text(
        Pos2::new(rect.right() - 48.0, rect.center().y),
        Align2::RIGHT_CENTER,
        "High Score",
        font,
        widgets::TEXT_SUBTLE,
    );
}

fn task_row(ui: &mut egui::Ui, name: &str, sub: &str, best: Option<i32>, selected: bool) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), ROW_H), Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let hovered = response.hovered();
    let p = ui.painter_at(rect);

    if hovered || selected {
        p.rect_filled(
            rect.shrink2(Vec2::new(0.0, 2.0)),
            egui::CornerRadius::same(8),
            widgets::SURFACE_HOVER,
        );
    }
    if selected {
        p.rect_filled(
            Rect::from_min_size(
                rect.left_top() + Vec2::new(0.0, 10.0),
                Vec2::new(3.0, ROW_H - 20.0),
            ),
            egui::CornerRadius::same(1),
            widgets::TARGET_YELLOW,
        );
    } else if !hovered {
        p.line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(1.0, widgets::GRID),
        );
    }

    let text_clip = Rect::from_min_max(rect.min, Pos2::new(rect.right() - 120.0, rect.bottom()));
    let tp = ui.painter_at(text_clip);
    tp.text(
        Pos2::new(rect.left() + 12.0, rect.top() + 16.0),
        Align2::LEFT_CENTER,
        name,
        FontId::proportional(13.5),
        Color32::WHITE,
    );
    tp.text(
        Pos2::new(rect.left() + 12.0, rect.top() + 34.0),
        Align2::LEFT_CENTER,
        sub,
        FontId::proportional(11.0),
        widgets::TEXT_SUBTLE,
    );

    let (score, color) = match best {
        Some(s) => (widgets::thousands(s as i64), Color32::WHITE),
        None => ("—".to_string(), widgets::TEXT_WEAK),
    };
    p.text(
        Pos2::new(rect.right() - 48.0, rect.center().y),
        Align2::RIGHT_CENTER,
        score,
        FontId::monospace(13.5),
        color,
    );

    // Play affordance: dark-slate disc that lights up in target yellow on hover.
    let c = Pos2::new(rect.right() - 20.0, rect.center().y);
    let (disc, glyph) = if hovered {
        (widgets::TARGET_YELLOW, widgets::BG)
    } else {
        (widgets::SURFACE_2, widgets::TARGET_YELLOW)
    };
    p.circle_filled(c, 12.5, disc);
    p.add(egui::Shape::convex_polygon(
        vec![
            c + Vec2::new(-3.2, -5.0),
            c + Vec2::new(5.0, 0.0),
            c + Vec2::new(-3.2, 5.0),
        ],
        glyph,
        Stroke::NONE,
    ));

    response.clicked()
}
