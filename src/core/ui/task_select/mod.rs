use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::EguiContexts;
use bevy_egui::egui::{self, Align, Layout, Pos2, Rect, Vec2};

use crate::core::state::AppState;
use crate::core::storage::Scores;
use crate::core::task_api::{ScoringMode, SelectedTask, TaskInfo, TaskRegistry};
use crate::core::ui::UiAccent;
use crate::core::ui::menu::sidebar::{self, NavTab};
use crate::core::ui::widgets;

pub mod card;
use card::render_task_card;

const SIDEBAR_WIDTH: f32 = 200.0;
const SIDEBAR_NARROW: f32 = 168.0;
const SIDEBAR_GAP: f32 = 16.0;
const CARD_GAP: f32 = 14.0;
const CARD_HEIGHT: f32 = 120.0;

#[derive(SystemParam)]
pub struct TaskSelectData<'w, 's> {
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    registry: Res<'w, TaskRegistry>,
    selected: ResMut<'w, SelectedTask>,
    scores: Res<'w, Scores>,
    accent: Res<'w, UiAccent>,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterTab {
    #[default]
    All,
    Click,
    Tracking,
    Sniper,
}

pub fn task_select_ui(
    mut contexts: EguiContexts,
    data: TaskSelectData,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
    mut filter: Local<FilterTab>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let TaskSelectData {
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

    let active_filter = &mut *filter;

    widgets::page(ctx, "task_select_screen", size, |ui| {
        let inner = widgets::page_inner(size);
        let sidebar_width = if inner.x < 1050.0 {
            SIDEBAR_NARROW
        } else {
            SIDEBAR_WIDTH
        };

        let (page_rect, _) = ui.allocate_exact_size(inner, egui::Sense::hover());
        let sidebar_rect =
            Rect::from_min_size(page_rect.min, Vec2::new(sidebar_width, page_rect.height()));
        let content_width = (page_rect.width() - sidebar_width - SIDEBAR_GAP).max(0.0);
        let content_rect = Rect::from_min_size(
            Pos2::new(sidebar_rect.right() + SIDEBAR_GAP, page_rect.top()),
            Vec2::new(content_width, page_rect.height()),
        );

        // 1. Sidebar with Play / Tasks highlighted
        let mut sidebar_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(sidebar_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        sidebar_ui.set_clip_rect(page_rect);
        sidebar::render_sidebar(
            &mut sidebar_ui,
            NavTab::PlayTasks,
            accent,
            page_rect.height(),
            sidebar_width,
            &mut next_state,
            &mut exit,
        );

        // 2. Main Content Area
        let mut content_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        content_ui.set_clip_rect(content_rect);

        // Top Navigation / Header Bar
        render_top_header(&mut content_ui, active_filter, registry.0.len());
        content_ui.add_space(14.0);

        // Filter tasks
        let filtered_tasks: Vec<&TaskInfo> = registry
            .0
            .iter()
            .filter(|t| match *active_filter {
                FilterTab::All => true,
                FilterTab::Click => t.scoring == ScoringMode::Click && !t.is_sniper_mode,
                FilterTab::Tracking => t.scoring == ScoringMode::Tracking,
                FilterTab::Sniper => t.is_sniper_mode,
            })
            .collect();

        // Responsive grid calculation
        let available_w = content_ui.available_width();
        let cols = if available_w >= 1000.0 {
            3
        } else if available_w >= 600.0 {
            2
        } else {
            1
        };

        let card_w = ((available_w - (CARD_GAP * (cols - 1) as f32)) / cols as f32).max(180.0);
        let card_size = Vec2::new(card_w, CARD_HEIGHT);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(&mut content_ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(CARD_GAP, CARD_GAP);

                for chunk in filtered_tasks.chunks(cols) {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = CARD_GAP;
                        for task in chunk {
                            let is_chosen = selected.0 == task.id;
                            if render_task_card(ui, task, is_chosen, &scores, accent, card_size) {
                                selected.0 = task.id;
                                next_state.set(AppState::TimeSelect);
                            }
                        }
                    });
                }
            });
    });
}

fn render_top_header(
    ui: &mut egui::Ui,
    active_filter: &mut FilterTab,
    total_count: usize,
) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(
                egui::RichText::new("SCENARIO LIBRARY")
                    .size(17.0)
                    .strong()
                    .color(widgets::TEXT),
            );
            ui.label(
                egui::RichText::new(format!("{total_count} training tasks available"))
                    .size(11.5)
                    .color(widgets::TEXT_SUBTLE),
            );
        });

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Filter Pills on the right
            render_filter_chip(ui, "SNIPER", FilterTab::Sniper, active_filter);
            render_filter_chip(ui, "TRACKING", FilterTab::Tracking, active_filter);
            render_filter_chip(ui, "CLICKING", FilterTab::Click, active_filter);
            render_filter_chip(ui, "ALL", FilterTab::All, active_filter);
        });
    });
}

fn render_filter_chip(
    ui: &mut egui::Ui,
    label: &str,
    tab: FilterTab,
    active_tab: &mut FilterTab,
) {
    let is_active = *active_tab == tab;
    let (bg, text_color, border) = if is_active {
        (widgets::SURFACE_2, widgets::ACCENT_CYAN, widgets::ACCENT_CYAN)
    } else {
        (widgets::SURFACE, widgets::TEXT_WEAK, widgets::BORDER)
    };

    let btn = ui.add(
        egui::Button::new(
            egui::RichText::new(label)
                .size(11.0)
                .strong()
                .color(text_color),
        )
        .fill(bg)
        .stroke(egui::Stroke::new(1.0, border))
        .corner_radius(egui::CornerRadius::same(6))
        .min_size(Vec2::new(0.0, 26.0)),
    );

    if btn.clicked() {
        *active_tab = tab;
    }
}
