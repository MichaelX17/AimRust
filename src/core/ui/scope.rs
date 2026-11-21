use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use egui::epaint::Shape;
use egui::{Color32, Pos2, Stroke};

use crate::core::ads::AdsState;

/// System to render Chamber's HeadHunter holographic optic overlay when ADS is active.
pub fn scope_ui(
    ads: Res<AdsState>,
    mut contexts: EguiContexts,
) {
    if ads.progress <= 0.001 {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let center = ctx.content_rect().center();

    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("chamber_scope_overlay"),
    ));

    draw_chamber_headhunter_scope(&painter, center, ads.progress);
}

fn draw_chamber_headhunter_scope(painter: &egui::Painter, center: Pos2, progress: f32) {
    let alpha = progress.clamp(0.0, 1.0);
    let scale = 0.94 + 0.06 * progress;

    let cx = center.x;
    let cy = center.y;

    // --- Frame geometry ---
    let hw = 175.0 * scale;
    let hh = 105.0 * scale;
    let chamfer = 28.0 * scale;
    let notch_h = 16.0 * scale;
    let notch_d = 8.0 * scale;

    let vertices = vec![
        Pos2::new(cx - hw + chamfer, cy - hh),
        Pos2::new(cx + hw - chamfer, cy - hh),
        Pos2::new(cx + hw, cy - hh + chamfer),
        Pos2::new(cx + hw, cy - notch_h),
        Pos2::new(cx + hw - notch_d, cy - notch_h + 4.0 * scale),
        Pos2::new(cx + hw - notch_d, cy + notch_h - 4.0 * scale),
        Pos2::new(cx + hw, cy + notch_h),
        Pos2::new(cx + hw, cy + hh - chamfer),
        Pos2::new(cx + hw - chamfer, cy + hh),
        Pos2::new(cx - hw + chamfer, cy + hh),
        Pos2::new(cx - hw, cy + hh - chamfer),
        Pos2::new(cx - hw, cy + notch_h),
        Pos2::new(cx - hw + notch_d, cy + notch_h - 4.0 * scale),
        Pos2::new(cx - hw + notch_d, cy - notch_h + 4.0 * scale),
        Pos2::new(cx - hw, cy - notch_h),
        Pos2::new(cx - hw, cy - hh + chamfer),
    ];

    // Frame outer glow
    painter.add(Shape::closed_line(
        vertices.clone(),
        Stroke::new(
            4.5,
            Color32::from_rgba_unmultiplied(255, 160, 20, (55.0 * alpha) as u8),
        ),
    ));

    // Frame sharp line (golden amber)
    painter.add(Shape::closed_line(
        vertices,
        Stroke::new(
            2.2,
            Color32::from_rgba_unmultiplied(255, 185, 30, (230.0 * alpha) as u8),
        ),
    ));

    // --- Top red-orange indicator arc ---
    let arc_radius = 92.0 * scale;
    let arc_center = Pos2::new(cx, cy - 20.0 * scale);
    let arc_start_deg = -125.0f32;
    let arc_end_deg = -55.0f32;
    let segments = 16;
    let mut arc_pts = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let deg = arc_start_deg + (arc_end_deg - arc_start_deg) * t;
        let rad = deg.to_radians();
        arc_pts.push(Pos2::new(
            arc_center.x + arc_radius * rad.cos(),
            arc_center.y + arc_radius * rad.sin(),
        ));
    }

    // Arc glow
    painter.add(Shape::line(
        arc_pts.clone(),
        Stroke::new(
            5.0,
            Color32::from_rgba_unmultiplied(255, 55, 40, (70.0 * alpha) as u8),
        ),
    ));

    // Arc sharp line
    painter.add(Shape::line(
        arc_pts,
        Stroke::new(
            2.4,
            Color32::from_rgba_unmultiplied(255, 75, 45, (230.0 * alpha) as u8),
        ),
    ));

    // --- Corner accent brackets ---
    let corner_stroke = Stroke::new(
        1.8,
        Color32::from_rgba_unmultiplied(255, 195, 45, (170.0 * alpha) as u8),
    );
    let offset = 5.0 * scale;
    let clen = chamfer * 0.7;

    // Top-left corner accent
    painter.line_segment(
        [
            Pos2::new(cx - hw + clen - offset, cy - hh - offset),
            Pos2::new(cx - hw - offset, cy - hh + clen - offset),
        ],
        corner_stroke,
    );
    // Top-right corner accent
    painter.line_segment(
        [
            Pos2::new(cx + hw - clen + offset, cy - hh - offset),
            Pos2::new(cx + hw + offset, cy - hh + clen - offset),
        ],
        corner_stroke,
    );
    // Bottom-left corner accent
    painter.line_segment(
        [
            Pos2::new(cx - hw + clen - offset, cy + hh + offset),
            Pos2::new(cx - hw - offset, cy + hh - clen + offset),
        ],
        corner_stroke,
    );
    // Bottom-right corner accent
    painter.line_segment(
        [
            Pos2::new(cx + hw - clen + offset, cy + hh + offset),
            Pos2::new(cx + hw + offset, cy + hh - clen + offset),
        ],
        corner_stroke,
    );

    // --- Side notch inner guide ticks ---
    let tick_stroke = Stroke::new(
        1.6,
        Color32::from_rgba_unmultiplied(255, 185, 30, (200.0 * alpha) as u8),
    );
    painter.line_segment(
        [
            Pos2::new(cx - hw + notch_d, cy),
            Pos2::new(cx - hw + notch_d + 14.0 * scale, cy),
        ],
        tick_stroke,
    );
    painter.line_segment(
        [
            Pos2::new(cx + hw - notch_d, cy),
            Pos2::new(cx + hw - notch_d - 14.0 * scale, cy),
        ],
        tick_stroke,
    );

    // --- Center reticle ---
    // Outer glow dot
    painter.circle_filled(
        center,
        3.5,
        Color32::from_rgba_unmultiplied(255, 140, 20, (120.0 * alpha) as u8),
    );
    // Crisp core dot
    painter.circle_filled(
        center,
        1.5,
        Color32::from_rgba_unmultiplied(255, 245, 190, (255.0 * alpha) as u8),
    );

    // Crosshair ticks
    let reticle_stroke = Stroke::new(
        1.6,
        Color32::from_rgba_unmultiplied(255, 210, 60, (220.0 * alpha) as u8),
    );
    painter.line_segment(
        [
            Pos2::new(cx - 16.0 * scale, cy),
            Pos2::new(cx - 6.0 * scale, cy),
        ],
        reticle_stroke,
    );
    painter.line_segment(
        [
            Pos2::new(cx + 6.0 * scale, cy),
            Pos2::new(cx + 16.0 * scale, cy),
        ],
        reticle_stroke,
    );
    painter.line_segment(
        [
            Pos2::new(cx, cy - 16.0 * scale),
            Pos2::new(cx, cy - 6.0 * scale),
        ],
        reticle_stroke,
    );

    // Lower optic tech chevron (Chamber HeadHunter style)
    let chevron_stroke = Stroke::new(
        1.4,
        Color32::from_rgba_unmultiplied(255, 195, 50, (180.0 * alpha) as u8),
    );
    painter.line_segment(
        [
            Pos2::new(cx - 5.0 * scale, cy + 11.0 * scale),
            Pos2::new(cx, cy + 8.0 * scale),
        ],
        chevron_stroke,
    );
    painter.line_segment(
        [
            Pos2::new(cx, cy + 8.0 * scale),
            Pos2::new(cx + 5.0 * scale, cy + 11.0 * scale),
        ],
        chevron_stroke,
    );
    painter.line_segment(
        [
            Pos2::new(cx - 3.0 * scale, cy + 16.0 * scale),
            Pos2::new(cx + 3.0 * scale, cy + 16.0 * scale),
        ],
        chevron_stroke,
    );
}
