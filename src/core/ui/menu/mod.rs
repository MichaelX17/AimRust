pub mod accuracy_card;
pub mod chart;
pub mod components;
pub mod header;
pub mod hero_chart;
pub mod leaderboard;
pub mod sidebar;
pub mod skill_breakdown;
pub mod stats;
pub mod targets_card;
pub mod task_roster;
pub mod twin_cards;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::EguiContexts;
use bevy_egui::egui::{self, Align, Layout, Pos2, Rect, Vec2};

use crate::core::round::RoundDuration;
use crate::core::state::AppState;
use crate::core::storage::Scores;
use crate::core::task_api::{SelectedTask, TaskRegistry};
use crate::core::ui::UiAccent;
use crate::core::ui::widgets;
use sidebar::NavTab;
use stats::DashStats;

const SIDEBAR_WIDTH: f32 = 200.0;
const SIDEBAR_NARROW: f32 = 168.0;
const SIDEBAR_GAP: f32 = 16.0;
const CARD_GAP: f32 = 14.0;
const STACK_BREAKPOINT: f32 = 720.0;

#[derive(SystemParam)]
pub struct HomeData<'w, 's> {
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    registry: Res<'w, TaskRegistry>,
    selected: ResMut<'w, SelectedTask>,
    duration: ResMut<'w, RoundDuration>,
    scores: Res<'w, Scores>,
    accent: Res<'w, UiAccent>,
}

pub fn menu_ui(
    mut contexts: EguiContexts,
    data: HomeData,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
    mut search_query: Local<String>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let HomeData {
        windows,
        registry,
        mut selected,
        mut duration,
        scores,
        accent,
    } = data;
    let Ok(window) = windows.single() else {
        return;
    };

    let accent = accent.0;
    let size = crate::core::ui::page_size(ctx, window);
    let active_tab = sidebar::active_dashboard_tab(ctx);
    let view_title = match active_tab {
        NavTab::Leaderboard => "Leaderboard",
        _ => "Dashboard",
    };

    let stats = DashStats::from_scores(&scores);
    widgets::page(ctx, "home_dashboard", size, |ui| {
        let inner = widgets::page_inner(size);
        let sidebar_width = if inner.x < 1050.0 {
            SIDEBAR_NARROW
        } else {
            SIDEBAR_WIDTH
        };

        // Layout mapping matching 01-MAIN_REFERENCE.jpg with exact rects
        let (page_rect, _) = ui.allocate_exact_size(inner, egui::Sense::hover());
        let sidebar_rect =
            Rect::from_min_size(page_rect.min, Vec2::new(sidebar_width, page_rect.height()));
        let content_width = (page_rect.width() - sidebar_width - SIDEBAR_GAP).max(0.0);
        let content_rect = Rect::from_min_size(
            Pos2::new(sidebar_rect.right() + SIDEBAR_GAP, page_rect.top()),
            Vec2::new(content_width, page_rect.height()),
        );

        // 1. Left Navigation Sidebar
        let mut sidebar_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(sidebar_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        sidebar_ui.set_clip_rect(page_rect);
        sidebar::render_sidebar(
            &mut sidebar_ui,
            active_tab,
            accent,
            page_rect.height(),
            sidebar_width,
            &mut next_state,
            &mut exit,
        );

        // 2. Main Content Dashboard Container
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
                // Top Header Bar: Title, Quick-Search, Notification bell, Avatar
                header::render_header(ui, view_title, &mut search_query, &stats);
                ui.add_space(12.0);

                if active_tab == NavTab::Leaderboard {
                    leaderboard::render_leaderboard_view(
                        ui,
                        leaderboard::LeaderboardParams {
                            registry: &registry,
                            selected: &mut selected,
                            duration: &mut duration,
                            scores: &scores,
                            search_query: &search_query,
                            next_state: &mut next_state,
                        },
                    );
                } else {
                    // Bento Grid: 2-Column Asymmetric Split
                    render_bento_grid(
                        ui,
                        &registry,
                        &mut selected,
                        &mut duration,
                        &scores,
                        &stats,
                        &search_query,
                        &mut next_state,
                    );
                }
            });
    });
}

#[allow(clippy::too_many_arguments)]
fn render_bento_grid(
    ui: &mut egui::Ui,
    registry: &TaskRegistry,
    selected: &mut SelectedTask,
    duration: &mut RoundDuration,
    scores: &Scores,
    stats: &DashStats,
    search_query: &str,
    next_state: &mut NextState<AppState>,
) {
    let total_width = ui.available_width();
    let available_height = ui.available_height();
    if total_width < 32.0 || available_height < 32.0 {
        return;
    }

    let grid_height = (available_height - 8.0).max(520.0);
    let row_height = ((grid_height - CARD_GAP) * 0.5).max(240.0);
    let top_height = row_height;
    let bottom_height = row_height;

    if total_width < STACK_BREAKPOINT {
        let wide = |h: f32| Vec2::new(total_width, h);
        hero_chart::render_hero_chart(
            ui,
            wide(top_height),
            hero_chart::HeroChartParams {
                selected,
                duration,
                scores,
                stats,
                next_state,
            },
        );
        ui.add_space(CARD_GAP);
        task_roster::render_task_roster(
            ui,
            wide(top_height),
            task_roster::TaskRosterParams {
                registry,
                selected,
                duration,
                scores,
                search_query,
                next_state,
            },
        );
        ui.add_space(CARD_GAP);
        twin_cards::render_twin_cards(ui, wide(bottom_height), CARD_GAP, stats);
        ui.add_space(CARD_GAP);
        skill_breakdown::render_skill_breakdown(ui, wide(bottom_height), stats);
        return;
    }

    // Asymmetric split matching 01-MAIN_REFERENCE.jpg (~63.5% left, ~36.5% right).
    let left_width = ((total_width - CARD_GAP) * 0.635).max(100.0);
    let right_width = (total_width - CARD_GAP - left_width).max(100.0);
    let (grid, _) = ui.allocate_exact_size(
        Vec2::new(total_width, top_height + CARD_GAP + bottom_height),
        egui::Sense::hover(),
    );
    let cell = |x: f32, y: f32, w: f32, h: f32| {
        Rect::from_min_size(Pos2::new(grid.left() + x, grid.top() + y), Vec2::new(w, h))
    };
    let right_x = left_width + CARD_GAP;
    let bottom_y = top_height + CARD_GAP;

    place_sub_ui(ui, cell(0.0, 0.0, left_width, top_height), |ui| {
        hero_chart::render_hero_chart(
            ui,
            Vec2::new(left_width, top_height),
            hero_chart::HeroChartParams {
                selected,
                duration,
                scores,
                stats,
                next_state,
            },
        );
    });
    place_sub_ui(ui, cell(right_x, 0.0, right_width, top_height), |ui| {
        task_roster::render_task_roster(
            ui,
            Vec2::new(right_width, top_height),
            task_roster::TaskRosterParams {
                registry,
                selected,
                duration,
                scores,
                search_query,
                next_state,
            },
        );
    });
    place_sub_ui(ui, cell(0.0, bottom_y, left_width, bottom_height), |ui| {
        twin_cards::render_twin_cards(ui, Vec2::new(left_width, bottom_height), CARD_GAP, stats);
    });
    place_sub_ui(ui, cell(right_x, bottom_y, right_width, bottom_height), |ui| {
        skill_breakdown::render_skill_breakdown(ui, Vec2::new(right_width, bottom_height), stats);
    });
}

fn place_sub_ui(ui: &mut egui::Ui, rect: Rect, add: impl FnOnce(&mut egui::Ui)) {
    let clip = ui.clip_rect().intersect(rect);
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::top_down(Align::Min)),
    );
    child.set_clip_rect(clip);
    add(&mut child);
}
