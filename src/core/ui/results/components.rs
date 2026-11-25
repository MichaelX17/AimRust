use bevy_egui::egui::{self, Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

use crate::core::ui::widgets;

#[derive(Clone, Copy)]
pub enum ResultsIcon {
    Crosshair,
    Activity,
    Summary,
}

pub fn results_icon(ui: &mut egui::Ui, icon: ResultsIcon, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::hover());
    let p = ui.painter_at(rect);
    let c = rect.center();
    let s = Stroke::new(1.7, color);
    let v = |x: f32, y: f32| c + Vec2::new(x, y);

    match icon {
        ResultsIcon::Crosshair => {
            p.circle_stroke(c, 8.0, s);
            p.circle_filled(c, 1.8, color);
            p.line_segment([v(-9.0, 0.0), v(-5.0, 0.0)], s);
            p.line_segment([v(5.0, 0.0), v(9.0, 0.0)], s);
            p.line_segment([v(0.0, -9.0), v(0.0, -5.0)], s);
            p.line_segment([v(0.0, 5.0), v(0.0, 9.0)], s);
        }
        ResultsIcon::Activity => {
            p.add(egui::Shape::line(
                vec![
                    v(-8.0, 2.0),
                    v(-4.0, 2.0),
                    v(-1.5, -5.5),
                    v(1.5, 6.0),
                    v(4.0, -2.0),
                    v(8.0, -2.0),
                ],
                s,
            ));
        }
        ResultsIcon::Summary => {
            for (x, h) in [(-5.5, 6.0), (0.0, 12.0), (5.5, 9.0)] {
                p.rect_filled(
                    Rect::from_min_size(v(x - 1.8, 6.0 - h), Vec2::new(3.6, h)),
                    CornerRadius::same(1),
                    color,
                );
            }
        }
    }
}

pub fn card_header(
    ui: &mut egui::Ui,
    icon: ResultsIcon,
    icon_color: Color32,
    title: &str,
    subtitle: Option<&str>,
    right_metric: Option<(&str, Color32)>,
) {
    ui.horizontal(|ui| {
        results_icon(ui, icon, icon_color);
        ui.add_space(4.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.label(
                egui::RichText::new(title)
                    .size(14.0)
                    .strong()
                    .color(widgets::TEXT),
            );
            if let Some(sub) = subtitle {
                ui.label(
                    egui::RichText::new(sub)
                        .size(11.0)
                        .color(widgets::TEXT_SUBTLE),
                );
            }
        });
        if let Some((metric, color)) = right_metric {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(metric)
                        .monospace()
                        .size(17.0)
                        .strong()
                        .color(color),
                );
            });
        }
    });
}

/// Vertical accent-bar resume metric tile matching the Dashboard design system.
pub fn accent_metric_tile(
    ui: &mut egui::Ui,
    label: &str,
    value: &str,
    sub: &str,
    accent: Color32,
    size: Vec2,
) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let p = ui.painter_at(rect);

    p.rect_filled(rect, CornerRadius::same(10), widgets::SURFACE_2);
    p.rect_stroke(
        rect,
        CornerRadius::same(10),
        Stroke::new(1.0, widgets::BORDER),
        StrokeKind::Inside,
    );

    let x = rect.left() + 12.0;
    p.text(
        Pos2::new(x, rect.top() + 15.0),
        Align2::LEFT_CENTER,
        label.to_uppercase(),
        FontId::proportional(10.0),
        widgets::TEXT_WEAK,
    );

    // Vertical accent bar immediately left of the hero tabular numeral
    let bar = Rect::from_min_size(Pos2::new(x, rect.top() + 29.0), Vec2::new(3.0, 22.0));
    p.rect_filled(bar, CornerRadius::same(2), accent);

    p.text(
        Pos2::new(x + 10.0, rect.top() + 40.0),
        Align2::LEFT_CENTER,
        value,
        FontId::monospace(20.0),
        Color32::WHITE,
    );

    p.text(
        Pos2::new(x, rect.bottom() - 12.0),
        Align2::LEFT_CENTER,
        sub,
        FontId::proportional(10.5),
        widgets::TEXT_SUBTLE,
    );
}
