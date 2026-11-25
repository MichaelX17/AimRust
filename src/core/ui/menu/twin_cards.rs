use bevy_egui::egui::{self, Align, Layout, Pos2, Rect, Vec2};

use super::accuracy_card::render_accuracy_card;
use super::stats::DashStats;
use super::targets_card::render_targets_card;

/// Bottom-left pair: accuracy trend and targets destroyed, split 50/50.
pub fn render_twin_cards(ui: &mut egui::Ui, size: Vec2, gap: f32, stats: &DashStats) {
    let half = Vec2::new(((size.x - gap) * 0.5).max(10.0), size.y);
    let (area, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let left = Rect::from_min_size(area.min, half);
    let right = Rect::from_min_size(Pos2::new(left.right() + gap, area.top()), half);

    let mut left_ui = ui.new_child(
        egui::UiBuilder::new().max_rect(left).layout(Layout::top_down(Align::Min)),
    );
    render_accuracy_card(&mut left_ui, half, stats);
    let mut right_ui = ui.new_child(
        egui::UiBuilder::new().max_rect(right).layout(Layout::top_down(Align::Min)),
    );
    render_targets_card(&mut right_ui, half, stats);
}
