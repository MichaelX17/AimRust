use bevy::prelude::*;

use crate::core::round::RoundTimer;
use crate::core::score::{RoundScore, scoring_mode};
use crate::core::state::AppState;
use crate::core::task_api::{ScoringMode, SelectedTask, TaskRegistry};

#[derive(Component)]
struct HudRoot;

#[derive(Component, Clone, Copy)]
enum HudField {
    Timer,
    Score,
    Accuracy,
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Countdown), spawn_hud)
            .add_systems(Update, update_hud)
            .add_systems(OnEnter(AppState::Results), despawn_hud)
            .add_systems(OnEnter(AppState::Menu), despawn_hud);
    }
}

fn hud_text(text: &str, field: HudField) -> impl Bundle {
    (
        Text::new(text),
        TextFont::from_font_size(26.0),
        TextColor(Color::WHITE),
        TextShadow {
            offset: Vec2::splat(2.0),
            color: Color::srgba(0.0, 0.0, 0.0, 0.85),
        },
        field,
    )
}

fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Start,
                padding: UiRect::axes(Val::Px(18.0), Val::Px(12.0)),
                ..default()
            },
            HudRoot,
        ))
        .with_children(|root| {
            root.spawn(hud_text("00:00", HudField::Timer));
            root.spawn(hud_text("0", HudField::Score));
            root.spawn(hud_text("0%", HudField::Accuracy));
        });
}

fn despawn_hud(mut commands: Commands, roots: Query<Entity, With<HudRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

/// Formats whole seconds as `mm:ss`.
pub fn format_time(seconds: u32) -> String {
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

fn update_hud(
    timer: Res<RoundTimer>,
    score: Res<RoundScore>,
    registry: Res<TaskRegistry>,
    selected: Res<SelectedTask>,
    mut fields: Query<(&HudField, &mut Text)>,
) {
    if fields.is_empty() {
        return;
    }
    let mode = scoring_mode(&registry, &selected);
    let seconds = timer.remaining.ceil().max(0.0) as u32;

    for (field, mut text) in &mut fields {
        text.0 = match field {
            HudField::Timer => format_time(seconds),
            HudField::Score => score.points.max(0).to_string(),
            HudField::Accuracy => match mode {
                ScoringMode::Click => format!("{:.0}%", crate::core::score::click_accuracy(&score)),
                ScoringMode::Tracking => format!("{} ms", score.tracking_ms.floor() as u64),
            },
        };
    }
}
