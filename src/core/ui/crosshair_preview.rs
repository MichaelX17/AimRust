use bevy_egui::egui;

use crate::core::config::{CrosshairCfg, CrosshairStyle, Settings};
use crate::core::ui::widgets;

#[allow(dead_code)]
pub fn crosshair_controls(ui: &mut egui::Ui, settings: &mut Settings) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            egui::Grid::new("crosshair_grid")
                .num_columns(4)
                .spacing([10.0, 10.0])
                .show(ui, |ui| {
                    row_label(ui, "Size");
                    ui.add(egui::Slider::new(&mut settings.crosshair.size, 1.0..=40.0));
                    row_label(ui, "Style");
                    egui::ComboBox::from_id_salt("crosshair_style")
                        .selected_text(format!("{:?}", settings.crosshair.style))
                        .show_ui(ui, |ui| {
                            for style in [
                                CrosshairStyle::Dot,
                                CrosshairStyle::Cross,
                                CrosshairStyle::Circle,
                                CrosshairStyle::T,
                            ] {
                                ui.selectable_value(
                                    &mut settings.crosshair.style,
                                    style,
                                    format!("{style:?}"),
                                );
                            }
                        });
                    ui.end_row();

                    row_label(ui, "Thickness");
                    ui.add(egui::Slider::new(
                        &mut settings.crosshair.thickness,
                        1.0..=10.0,
                    ));
                    row_label(ui, "Color");
                    ui.color_edit_button_srgb(&mut settings.crosshair.color);
                    ui.end_row();

                    row_label(ui, "Gap");
                    ui.add(egui::Slider::new(&mut settings.crosshair.gap, 0.0..=30.0));
                    row_label(ui, "Outline");
                    ui.checkbox(&mut settings.crosshair.outline, "");
                    ui.end_row();
                });
        });

        ui.add_space(8.0);
        render_crosshair_preview_box(ui, &settings.crosshair);
    });
}

#[allow(dead_code)]
pub fn render_crosshair_preview_box(ui: &mut egui::Ui, cfg: &CrosshairCfg) {
    render_crosshair_preview_box_sized(ui, cfg, 76.0);
}

pub fn render_crosshair_preview_box_sized(ui: &mut egui::Ui, cfg: &CrosshairCfg, side: f32) {
    let box_size = egui::vec2(side, side);
    let (rect, _) = ui.allocate_exact_size(box_size, egui::Sense::hover());
    let painter = ui.painter();

    // Dark inset preview box with subtle border
    painter.rect_filled(rect, egui::CornerRadius::same(8), widgets::SURFACE_2);
    painter.rect_stroke(
        rect,
        egui::CornerRadius::same(8),
        egui::Stroke::new(1.0, widgets::BORDER),
        egui::StrokeKind::Inside,
    );

    // Clip crosshair drawing to the inner bounds so large size/gap values never bleed out
    let painter = ui.painter_at(rect.shrink(2.0));
    let center = rect.center();
    let col = egui::Color32::from_rgb(cfg.color[0], cfg.color[1], cfg.color[2]);
    let outline_col = egui::Color32::BLACK;
    let outline = cfg.outline;

    let draw_bar = |painter: &egui::Painter, p_min: egui::Pos2, p_max: egui::Pos2| {
        let bar_rect = egui::Rect::from_min_max(p_min, p_max);
        if outline {
            let out_rect = bar_rect.expand(1.0);
            painter.rect_filled(out_rect, egui::CornerRadius::ZERO, outline_col);
        }
        painter.rect_filled(bar_rect, egui::CornerRadius::ZERO, col);
    };

    let size = cfg.size.max(1.0);
    let thick = cfg.thickness.max(1.0);
    let gap = cfg.gap.max(0.0);
    let half_thick = thick * 0.5;

    match cfg.style {
        CrosshairStyle::Dot => {
            draw_bar(
                &painter,
                center - egui::vec2(half_thick, half_thick),
                center + egui::vec2(half_thick, half_thick),
            );
        }
        CrosshairStyle::Cross => {
            // Top
            draw_bar(
                &painter,
                egui::pos2(center.x - half_thick, center.y - gap - size),
                egui::pos2(center.x + half_thick, center.y - gap),
            );
            // Bottom
            draw_bar(
                &painter,
                egui::pos2(center.x - half_thick, center.y + gap),
                egui::pos2(center.x + half_thick, center.y + gap + size),
            );
            // Left
            draw_bar(
                &painter,
                egui::pos2(center.x - gap - size, center.y - half_thick),
                egui::pos2(center.x - gap, center.y + half_thick),
            );
            // Right
            draw_bar(
                &painter,
                egui::pos2(center.x + gap, center.y - half_thick),
                egui::pos2(center.x + gap + size, center.y + half_thick),
            );
        }
        CrosshairStyle::Circle => {
            if outline {
                painter.circle_stroke(
                    center,
                    size * 0.5,
                    egui::Stroke::new(thick + 2.0, outline_col),
                );
            }
            painter.circle_stroke(center, size * 0.5, egui::Stroke::new(thick, col));
        }
        CrosshairStyle::T => {
            // Bottom
            draw_bar(
                &painter,
                egui::pos2(center.x - half_thick, center.y + gap),
                egui::pos2(center.x + half_thick, center.y + gap + size),
            );
            // Left
            draw_bar(
                &painter,
                egui::pos2(center.x - gap - size, center.y - half_thick),
                egui::pos2(center.x - gap, center.y + half_thick),
            );
            // Right
            draw_bar(
                &painter,
                egui::pos2(center.x + gap, center.y - half_thick),
                egui::pos2(center.x + gap + size, center.y + half_thick),
            );
        }
    }
}

#[allow(dead_code)]
fn row_label(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .size(13.0)
            .color(widgets::TEXT_WEAK),
    );
}
