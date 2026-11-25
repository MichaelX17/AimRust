use bevy_egui::egui::{
    self, Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2,
};

use crate::core::hud::format_time;
use crate::core::round::RoundTimer;
use crate::core::score::RoundScore;
use crate::core::task_api::ScoringMode;
use crate::core::ui::widgets;

pub struct SessionCardParams<'a> {
    pub task_name: &'a str,
    pub duration: u32,
    pub mode: ScoringMode,
    pub score: &'a RoundScore,
    pub timer: &'a RoundTimer,
    pub highest_score: i32,
    pub new_best: bool,
    pub doa_failed: bool,
}

/// Top hero banner card prominently displaying the Achieved Score and Highest Score.
pub fn render_session_card(ui: &mut egui::Ui, size: Vec2, params: SessionCardParams) {
    let SessionCardParams {
        task_name,
        duration,
        mode,
        score,
        timer,
        highest_score,
        new_best,
        doa_failed,
    } = params;

    widgets::fixed_card(ui, size, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;

        // Top meta row: status chip + task name/duration + grade badge
        render_top_meta(ui, task_name, duration, mode, score, timer.duration, new_best, doa_failed);
        ui.add_space(12.0);

        let area = Vec2::new(ui.available_width(), ui.available_height().max(76.0));
        let (rect, _) = ui.allocate_exact_size(area, Sense::hover());
        let p = ui.painter_at(rect);

        let achieved_raw = match mode {
            ScoringMode::Click => score.points.max(0),
            ScoringMode::Tracking => score.tracking_ms.floor() as i32,
        };
        let highest_raw = highest_score.max(achieved_raw);

        let achieved_str = match mode {
            ScoringMode::Click => widgets::thousands(achieved_raw as i64),
            ScoringMode::Tracking => format!("{} ms", achieved_raw),
        };
        let highest_str = match mode {
            ScoringMode::Click => widgets::thousands(highest_raw as i64),
            ScoringMode::Tracking => format!("{} ms", highest_raw),
        };

        let gap = 14.0;
        let half_w = ((rect.width() - gap) * 0.5).max(120.0);
        let left_rect = Rect::from_min_size(rect.min, Vec2::new(half_w, rect.height()));
        let right_rect = Rect::from_min_size(
            Pos2::new(left_rect.right() + gap, rect.top()),
            Vec2::new(half_w, rect.height()),
        );

        // Left block: Achieved Score
        draw_hero_score_block(
            &p,
            left_rect,
            "ACHIEVED SCORE",
            &achieved_str,
            Color32::WHITE,
            widgets::ACCENT_CYAN,
            delta_badge(achieved_raw, highest_raw, new_best),
        );

        // Right block: Highest Score
        draw_hero_score_block(
            &p,
            right_rect,
            "HIGHEST SCORE",
            &highest_str,
            widgets::TARGET_YELLOW,
            widgets::TARGET_YELLOW,
            Some((
                if new_best {
                    "★ NEW RECORD".to_string()
                } else {
                    "PERSONAL BEST".to_string()
                },
                widgets::TARGET_YELLOW,
            )),
        );
    });
}

#[allow(clippy::too_many_arguments)]
fn render_top_meta(
    ui: &mut egui::Ui,
    task_name: &str,
    duration: u32,
    mode: ScoringMode,
    score: &RoundScore,
    round_dur: f32,
    new_best: bool,
    doa_failed: bool,
) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 28.0), Sense::hover());
    let p = ui.painter_at(rect);

    let (status_text, status_color) = if doa_failed {
        ("HARDCORE FAILED (MISS)", widgets::NEGATIVE)
    } else if new_best {
        ("NEW PERSONAL BEST", widgets::POSITIVE)
    } else {
        ("ROUND COMPLETE", widgets::TARGET_YELLOW)
    };

    let chip_w = if doa_failed { 174.0 } else { 150.0 };
    let chip_rect = Rect::from_min_size(Pos2::new(rect.left(), rect.center().y - 12.0), Vec2::new(chip_w, 24.0));
    p.rect_filled(chip_rect, CornerRadius::same(6), status_color.gamma_multiply(0.16));
    p.rect_stroke(
        chip_rect,
        CornerRadius::same(6),
        Stroke::new(1.0, status_color.gamma_multiply(0.6)),
        StrokeKind::Inside,
    );
    p.text(
        chip_rect.center(),
        Align2::CENTER_CENTER,
        status_text,
        FontId::monospace(10.5),
        status_color,
    );

    p.text(
        Pos2::new(chip_rect.right() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        format!("{task_name}  ·  {}", format_time(duration)),
        FontId::monospace(12.5),
        widgets::TEXT_SUBTLE,
    );

    // Compact grade pill on the far right
    let (grade, grade_color) = calculate_grade(mode, score, round_dur);
    let grade_rect = Rect::from_min_size(
        Pos2::new(rect.right() - 78.0, rect.center().y - 12.0),
        Vec2::new(78.0, 24.0),
    );
    p.rect_filled(grade_rect, CornerRadius::same(6), widgets::SURFACE_2);
    p.rect_stroke(
        grade_rect,
        CornerRadius::same(6),
        Stroke::new(1.0, grade_color.gamma_multiply(0.75)),
        StrokeKind::Inside,
    );
    p.text(
        grade_rect.center(),
        Align2::CENTER_CENTER,
        format!("RANK {grade}"),
        FontId::monospace(11.0),
        grade_color,
    );
}

fn draw_hero_score_block(
    p: &egui::Painter,
    rect: Rect,
    label: &str,
    value: &str,
    value_color: Color32,
    accent: Color32,
    badge: Option<(String, Color32)>,
) {
    p.rect_filled(rect, CornerRadius::same(10), widgets::SURFACE_2);
    p.rect_stroke(
        rect,
        CornerRadius::same(10),
        Stroke::new(1.0, widgets::BORDER),
        StrokeKind::Inside,
    );

    // Vertical accent bar on left edge
    p.rect_filled(
        Rect::from_min_size(rect.min + Vec2::new(12.0, 14.0), Vec2::new(3.5, rect.height() - 28.0)),
        CornerRadius::same(2),
        accent,
    );

    let x = rect.left() + 26.0;
    p.text(
        Pos2::new(x, rect.top() + 18.0),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(10.5),
        widgets::TEXT_WEAK,
    );

    p.text(
        Pos2::new(x, rect.bottom() - 26.0),
        Align2::LEFT_CENTER,
        value,
        FontId::monospace(32.0),
        value_color,
    );

    if let Some((text, color)) = badge {
        p.text(
            Pos2::new(rect.right() - 14.0, rect.top() + 18.0),
            Align2::RIGHT_CENTER,
            text,
            FontId::monospace(10.5),
            color,
        );
    }
}

fn delta_badge(achieved: i32, highest: i32, new_best: bool) -> Option<(String, Color32)> {
    if new_best {
        return Some(("▲ NEW HIGH".to_string(), widgets::POSITIVE));
    }
    if highest <= 0 {
        return None;
    }
    let diff = achieved - highest;
    let pct = (diff as f32 / highest as f32) * 100.0;
    if diff >= 0 {
        Some((format!("▲ +{pct:.1}% vs PB"), widgets::POSITIVE))
    } else {
        Some((format!("▼ {pct:.1}% vs PB"), widgets::NEGATIVE))
    }
}

fn calculate_grade(mode: ScoringMode, score: &RoundScore, duration: f32) -> (&'static str, Color32) {
    match mode {
        ScoringMode::Click => {
            let shots = score.hits + score.misses;
            let acc = if shots > 0 { score.hits as f32 / shots as f32 * 100.0 } else { 0.0 };
            let hps = if duration > 0.0 { score.hits as f32 / duration } else { 0.0 };
            if acc >= 95.0 && hps >= 2.0 {
                ("S", widgets::TARGET_YELLOW)
            } else if acc >= 90.0 && hps >= 1.5 {
                ("A", widgets::ACCENT_CYAN)
            } else if acc >= 80.0 {
                ("B", widgets::ACCENT_VIOLET)
            } else if acc >= 70.0 {
                ("C", widgets::TEXT_WEAK)
            } else {
                ("D", widgets::NEGATIVE)
            }
        }
        ScoringMode::Tracking => {
            let round_ms = duration as f64 * 1000.0;
            let ratio = if round_ms > 0.0 { (score.tracking_ms / round_ms * 100.0) as f32 } else { 0.0 };
            if ratio >= 85.0 {
                ("S", widgets::TARGET_YELLOW)
            } else if ratio >= 70.0 {
                ("A", widgets::ACCENT_CYAN)
            } else if ratio >= 55.0 {
                ("B", widgets::ACCENT_VIOLET)
            } else if ratio >= 40.0 {
                ("C", widgets::TEXT_WEAK)
            } else {
                ("D", widgets::NEGATIVE)
            }
        }
    }
}
