use bevy::prelude::*;
use bevy::window::Window;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

use crate::core::config::Settings;
use crate::core::state::AppState;

pub mod crosshair_preview;
pub mod menu;
pub mod pause;
pub mod results;
pub mod scope;
pub mod settings;
pub mod task_select;
pub mod time_select;
pub mod widgets;

/// Accent color derived from `Settings.colors.background_accent`.
#[derive(Resource)]
pub struct UiAccent(pub egui::Color32);

impl Default for UiAccent {
    fn default() -> Self {
        let [r, g, b] = crate::core::config::ColorsCfg::default().background_accent;
        Self(egui::Color32::from_rgb(r, g, b))
    }
}

/// Logical page size derived from the real window (not egui's cached screen
/// rect), so the layout stays responsive when the window mode changes live.
pub fn page_size(ctx: &egui::Context, window: &Window) -> egui::Vec2 {
    let pixels_per_point = ctx.pixels_per_point().max(0.1);
    egui::vec2(window.width(), window.height())
        * (window.scale_factor() / pixels_per_point)
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<UiAccent>()
            .init_resource::<pause::PauseUiState>()
            .init_resource::<pause::PendingRestart>()
            .add_systems(OnEnter(AppState::Paused), pause::reset_pause_state)
            .add_systems(
                EguiPrimaryContextPass,
                (
                    apply_theme,
                    menu::menu_ui.run_if(in_state(AppState::Menu)),
                    settings::settings_screen.run_if(in_state(AppState::Settings)),
                    task_select::task_select_ui.run_if(in_state(AppState::TaskSelect)),
                    time_select::time_select_ui.run_if(in_state(AppState::TimeSelect)),
                    pause::pause_ui.run_if(in_state(AppState::Paused)),
                    results::results_ui.run_if(in_state(AppState::Results)),
                    scope::scope_ui.run_if(in_gameplay),
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    pause::handle_pending_restart.run_if(in_state(AppState::Menu)),
                    pause::esc_pause.run_if(in_state(AppState::Playing)),
                    pause::esc_resume.run_if(in_state(AppState::Paused)),
                ),
            );
    }
}

fn in_gameplay(state: Res<State<AppState>>) -> bool {
    matches!(*state.get(), AppState::Playing | AppState::Countdown)
}

/// Applies the dashboard look from `Settings.colors.background_accent`; re-runs
/// only when the settings change.
fn apply_theme(
    settings: Res<Settings>,
    mut clear_color: ResMut<ClearColor>,
    mut contexts: EguiContexts,
    mut accent: ResMut<UiAccent>,
    mut applied: Local<bool>,
) {
    if *applied && !settings.is_changed() {
        return;
    }

    let [r, g, b] = settings.colors.background_accent;
    let bg = dark_tint([r, g, b]);
    clear_color.0 = Color::srgb_u8(bg[0], bg[1], bg[2]);
    accent.0 = egui::Color32::from_rgb(r, g, b);

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    ctx.set_visuals(theme_visuals([r, g, b], bg));
    ctx.all_styles_mut(|style| {
        use egui::{FontFamily, FontId, TextStyle};
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(26.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Body,
            FontId::new(15.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(15.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(13.0, FontFamily::Monospace),
        );
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(14.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(16);
        style.spacing.menu_margin = egui::Margin::same(10);
        style.spacing.interact_size = egui::vec2(42.0, 32.0);
    });
    *applied = true;
}

/// Very dark shade of the accent, used for the scene clear color and page fill.
fn dark_tint(accent: [u8; 3]) -> [u8; 3] {
    [
        (9.0 + accent[0] as f32 * 0.03) as u8,
        (9.0 + accent[1] as f32 * 0.03) as u8,
        (14.0 + accent[2] as f32 * 0.04) as u8,
    ]
}

fn theme_visuals(accent: [u8; 3], bg: [u8; 3]) -> egui::Visuals {
    use egui::{Color32, CornerRadius, Stroke};

    let accent = Color32::from_rgb(accent[0], accent[1], accent[2]);
    let bg = Color32::from_rgb(bg[0], bg[1], bg[2]);
    let text = Color32::from_rgb(232, 232, 244);
    let weak = Color32::from_rgb(142, 142, 164);
    let surface = Color32::from_rgb(17, 17, 26);
    let surface_2 = Color32::from_rgb(25, 25, 38);
    let border = Color32::from_rgb(38, 38, 56);

    let mut visuals = egui::Visuals::dark();
    visuals.dark_mode = true;
    visuals.override_text_color = Some(text);
    visuals.panel_fill = bg;
    visuals.window_fill = surface;
    visuals.faint_bg_color = surface_2;
    visuals.extreme_bg_color = bg;
    visuals.code_bg_color = surface_2;
    visuals.text_edit_bg_color = Some(surface_2);
    visuals.window_corner_radius = CornerRadius::same(16);
    visuals.menu_corner_radius = CornerRadius::same(10);
    visuals.window_stroke = Stroke::new(1.0, border);
    visuals.selection.bg_fill = accent.gamma_multiply(0.35);
    visuals.selection.stroke = Stroke::new(1.0, accent);
    visuals.hyperlink_color = accent;

    visuals.widgets.noninteractive.bg_fill = surface;
    visuals.widgets.noninteractive.weak_bg_fill = surface;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, weak);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(10);

    visuals.widgets.inactive.bg_fill = surface_2;
    visuals.widgets.inactive.weak_bg_fill = surface_2;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(10);

    visuals.widgets.hovered.bg_fill = accent.gamma_multiply(0.25);
    visuals.widgets.hovered.weak_bg_fill = accent.gamma_multiply(0.25);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, accent.gamma_multiply(0.8));
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, text);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(10);

    visuals.widgets.active.bg_fill = accent.gamma_multiply(0.65);
    visuals.widgets.active.weak_bg_fill = accent.gamma_multiply(0.65);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, accent);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, text);
    visuals.widgets.active.corner_radius = CornerRadius::same(10);

    // Show a pointer hand over every interactive element (all menu buttons).
    visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);

    visuals
}
