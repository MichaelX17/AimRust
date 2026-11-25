use bevy::prelude::*;
use bevy_egui::egui::{self, Align, Align2, Color32, FontId, Layout, Pos2, Rect, Stroke, Vec2};

use crate::core::state::AppState;
use crate::core::ui::widgets;

const DASHBOARD_TAB_KEY: &str = "dashboard_active_tab";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NavTab {
    Dashboard,
    PlayTasks,
    Leaderboard,
    Settings,
    Exit,
}

pub fn active_dashboard_tab(ctx: &egui::Context) -> NavTab {
    ctx.data(|d| d.get_temp::<NavTab>(egui::Id::new(DASHBOARD_TAB_KEY)))
        .unwrap_or(NavTab::Dashboard)
}

pub fn render_sidebar(
    ui: &mut egui::Ui,
    active_tab: NavTab,
    accent: Color32,
    height: f32,
    width: f32,
    next_state: &mut NextState<AppState>,
    exit: &mut MessageWriter<AppExit>,
) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());
    let painter = ui.painter();

    // Fixed vertical dark-slate panel (#21252B / elevated dark surface)
    painter.rect_filled(rect, egui::CornerRadius::same(12), widgets::SURFACE_2);
    painter.rect_stroke(
        rect,
        egui::CornerRadius::same(12),
        Stroke::new(1.0, widgets::BORDER),
        egui::StrokeKind::Inside,
    );

    let mut inner = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(Vec2::new(12.0, 16.0)))
            .layout(Layout::top_down(Align::Min)),
    );

    // App Logo & Title
    render_brand(&mut inner);
    inner.add_space(20.0);

    // Primary navigation items
    let nav_items = [
        ("Dashboard", NavTab::Dashboard),
        ("Play / Tasks", NavTab::PlayTasks),
        ("Leaderboard", NavTab::Leaderboard),
        ("Settings", NavTab::Settings),
    ];

    inner.spacing_mut().item_spacing.y = 4.0;
    for (label, tab) in nav_items {
        if nav_button(&mut inner, label, tab, active_tab == tab, accent).clicked() {
            handle_nav_click(inner.ctx(), tab, active_tab, next_state);
        }
    }

    inner.add_space((inner.available_height() - 74.0).max(8.0));

    if nav_button(&mut inner, "Exit", NavTab::Exit, false, accent).clicked() {
        exit.write(AppExit::Success);
    }

    inner.add_space(10.0);

    // System Status Footer
    inner.horizontal(|ui| {
        let (dot, _) = ui.allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover());
        ui.painter()
            .circle_filled(dot.center(), 3.5, widgets::POSITIVE);
        ui.label(
            egui::RichText::new(format!("READY · v{}", env!("CARGO_PKG_VERSION")))
                .monospace()
                .size(10.0)
                .strong()
                .color(widgets::POSITIVE),
        );
    });
}

fn handle_nav_click(
    ctx: &egui::Context,
    tab: NavTab,
    active_tab: NavTab,
    next_state: &mut NextState<AppState>,
) {
    match tab {
        NavTab::Dashboard | NavTab::Leaderboard => {
            ctx.data_mut(|d| d.insert_temp(egui::Id::new(DASHBOARD_TAB_KEY), tab));
            if active_tab != tab {
                next_state.set(AppState::Menu);
            }
        }
        NavTab::PlayTasks => {
            next_state.set(AppState::TaskSelect);
        }
        NavTab::Settings => {
            if active_tab != NavTab::Settings {
                next_state.set(AppState::Settings);
            }
        }
        NavTab::Exit => {}
    }
}

fn render_brand(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(26.0), egui::Sense::hover());
        let p = ui.painter_at(icon_rect);
        let c = icon_rect.center();

        p.circle_filled(c, 13.0, widgets::BG);
        p.circle_stroke(c, 11.0, Stroke::new(1.5, widgets::TARGET_YELLOW));
        p.circle_stroke(c, 5.5, Stroke::new(1.5, widgets::ACCENT_CYAN));
        p.circle_filled(c, 2.0, Color32::WHITE);
        for (a, b) in [
            (Vec2::new(-10.0, 0.0), Vec2::new(-6.5, 0.0)),
            (Vec2::new(6.5, 0.0), Vec2::new(10.0, 0.0)),
            (Vec2::new(0.0, -10.0), Vec2::new(0.0, -6.5)),
            (Vec2::new(0.0, 6.5), Vec2::new(0.0, 10.0)),
        ] {
            p.line_segment([c + a, c + b], Stroke::new(1.5, widgets::TARGET_YELLOW));
        }

        ui.add_space(6.0);
        ui.label(
            egui::RichText::new("AimRust")
                .size(18.0)
                .strong()
                .color(widgets::TEXT),
        );
    });
}

fn nav_button(
    ui: &mut egui::Ui,
    label: &str,
    tab: NavTab,
    active: bool,
    _accent: Color32,
) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 38.0), egui::Sense::click());
    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let hovered = response.hovered();

    let painter = ui.painter();
    if active {
        // Distinct pill/card active highlight state matching 01-MAIN_REFERENCE.jpg
        painter.rect_filled(rect, egui::CornerRadius::same(8), widgets::SURFACE_HOVER);
        painter.rect_stroke(
            rect,
            egui::CornerRadius::same(8),
            Stroke::new(1.0, Color32::from_rgb(62, 70, 84)),
            egui::StrokeKind::Inside,
        );
        painter.rect_filled(
            Rect::from_min_size(rect.min + Vec2::new(1.0, 8.0), Vec2::new(3.0, 22.0)),
            egui::CornerRadius::same(2),
            widgets::TARGET_YELLOW,
        );
    } else if hovered {
        painter.rect_filled(
            rect,
            egui::CornerRadius::same(8),
            Color32::from_rgb(38, 43, 51),
        );
    }

    let icon_color = if active {
        Color32::WHITE
    } else if hovered {
        widgets::TEXT
    } else {
        widgets::TEXT_WEAK
    };
    let text_color = if active {
        Color32::WHITE
    } else if hovered {
        widgets::TEXT
    } else {
        widgets::TEXT_WEAK
    };

    draw_nav_icon(ui, tab, rect, icon_color);
    painter.text(
        Pos2::new(rect.left() + 40.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.5),
        text_color,
    );
    response
}

fn draw_nav_icon(ui: &egui::Ui, tab: NavTab, rect: Rect, color: Color32) {
    let p = ui.painter_at(rect);
    let s = Stroke::new(1.5, color);
    let c = Pos2::new(rect.left() + 20.0, rect.center().y);
    let v = |x: f32, y: f32| c + Vec2::new(x, y);

    match tab {
        NavTab::Dashboard => {
            for dx in [-4.5, 1.0] {
                for dy in [-4.5, 1.0] {
                    p.rect_filled(
                        Rect::from_min_size(v(dx, dy), Vec2::splat(3.5)),
                        egui::CornerRadius::same(1),
                        color,
                    );
                }
            }
        }
        NavTab::PlayTasks => {
            p.add(egui::Shape::convex_polygon(
                vec![v(-3.5, -5.5), v(5.5, 0.0), v(-3.5, 5.5)],
                color,
                Stroke::NONE,
            ));
        }
        NavTab::Leaderboard => {
            p.rect_stroke(
                Rect::from_center_size(v(0.0, -1.5), Vec2::new(8.0, 7.0)),
                egui::CornerRadius::same(2),
                s,
                egui::StrokeKind::Inside,
            );
            p.line_segment([v(0.0, 2.0), v(0.0, 5.5)], s);
            p.line_segment([v(-3.5, 5.5), v(3.5, 5.5)], s);
        }
        NavTab::Settings => {
            for offset in [-4.0, 0.0, 4.0] {
                p.line_segment([v(-5.5, offset), v(5.5, offset)], s);
            }
            p.circle_filled(v(-2.0, -4.0), 1.6, color);
            p.circle_filled(v(2.5, 0.0), 1.6, color);
            p.circle_filled(v(-1.0, 4.0), 1.6, color);
        }
        NavTab::Exit => {
            p.line_segment([v(-5.5, -5.5), v(-5.5, 5.5)], s);
            p.line_segment([v(-5.5, -5.5), v(1.5, -5.5)], s);
            p.line_segment([v(-5.5, 5.5), v(1.5, 5.5)], s);
            p.line_segment([v(-1.0, 0.0), v(5.5, 0.0)], s);
            p.line_segment([v(2.5, -3.0), v(5.5, 0.0)], s);
            p.line_segment([v(2.5, 3.0), v(5.5, 0.0)], s);
        }
    }
}
