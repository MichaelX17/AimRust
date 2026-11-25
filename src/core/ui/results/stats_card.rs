use bevy_egui::egui::{self, Color32, Vec2};

use super::components::{ResultsIcon, accent_metric_tile, card_header};
use crate::core::score::RoundScore;
use crate::core::storage::TaskScores;
use crate::core::ui::widgets;

const SHOTS_WHITE: Color32 = Color32::from_rgb(220, 232, 245);

/// Stylized Session Resume card showing:
/// - AVG points from last 5 scores
/// - AVG accuracy from last 5 scores
/// - Hits (current round)
/// - Shots (current round)
pub fn render_stats_card(
    ui: &mut egui::Ui,
    size: Vec2,
    score: &RoundScore,
    task_scores: Option<&TaskScores>,
) {
    let last_five = task_scores
        .map(|t| {
            let count = t.recent.len().min(5);
            &t.recent[t.recent.len() - count..]
        })
        .unwrap_or(&[]);

    let avg_points = if last_five.is_empty() {
        score.points.max(0) as f32
    } else {
        let sum: f32 = last_five.iter().map(|r| r.score.max(0) as f32).sum();
        sum / last_five.len() as f32
    };

    let avg_acc = if last_five.is_empty() {
        let total = score.hits + score.misses;
        if total > 0 {
            score.hits as f32 / total as f32 * 100.0
        } else {
            0.0
        }
    } else {
        let sum: f32 = last_five
            .iter()
            .map(|r| {
                let total = r.hits + r.misses;
                if total > 0 {
                    r.hits as f32 / total as f32 * 100.0
                } else {
                    0.0
                }
            })
            .sum();
        sum / last_five.len() as f32
    };

    let hits = score.hits;
    let shots = score.hits + score.misses;
    let sample_label = match last_five.len() {
        0 | 1 => "Current round".to_string(),
        n => format!("Last {n} rounds"),
    };

    widgets::fixed_card(ui, size, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        card_header(
            ui,
            ResultsIcon::Summary,
            widgets::TARGET_YELLOW,
            "Session Resume",
            Some("5-round averages & current round totals"),
            None,
        );
        ui.add_space(12.0);

        let gap = 10.0;
        let avail = ui.available_size();
        let tiles = [
            (
                "Avg Points (Last 5)",
                widgets::thousands(avg_points.round() as i64),
                sample_label.clone(),
                widgets::ACCENT_CYAN,
            ),
            (
                "Avg Accuracy (Last 5)",
                format!("{avg_acc:.1}%"),
                sample_label,
                widgets::TARGET_YELLOW,
            ),
            (
                "Hits",
                widgets::thousands(hits as i64),
                "Targets destroyed".to_string(),
                widgets::POSITIVE,
            ),
            (
                "Shots",
                widgets::thousands(shots as i64),
                format!("Misses: {}", score.misses),
                SHOTS_WHITE,
            ),
        ];

        // 4 columns on wide screens, 2x2 grid on narrow screens
        if avail.x >= 560.0 {
            let tile_w = ((avail.x - gap * 3.0) / 4.0).max(110.0);
            let tile_h = avail.y.max(72.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for (label, value, sub, color) in &tiles {
                    accent_metric_tile(ui, label, value, sub, *color, Vec2::new(tile_w, tile_h));
                }
            });
        } else {
            let tile_w = ((avail.x - gap) * 0.5).max(110.0);
            let tile_h = ((avail.y - gap) * 0.5).max(68.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = gap;
                for row in tiles.chunks(2) {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        for (label, value, sub, color) in row {
                            accent_metric_tile(
                                ui,
                                label,
                                value,
                                sub,
                                *color,
                                Vec2::new(tile_w, tile_h),
                            );
                        }
                    });
                }
            });
        }
    });
}
