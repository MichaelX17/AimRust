use bevy::prelude::*;

use crate::core::config::{CrosshairCfg, CrosshairStyle, Settings};
use crate::core::state::AppState;

/// Root of the crosshair UI tree.
#[derive(Component)]
pub struct CrosshairRoot;

pub struct CrosshairPlugin;

impl Plugin for CrosshairPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Countdown), spawn_crosshair)
            .add_systems(OnEnter(AppState::Settings), spawn_crosshair)
            .add_systems(Update, rebuild_crosshair)
            .add_systems(OnEnter(AppState::Results), despawn_crosshair)
            .add_systems(OnEnter(AppState::Menu), despawn_crosshair);
    }
}

fn spawn_crosshair(mut commands: Commands, settings: Res<Settings>) {
    spawn_crosshair_ui(&mut commands, &settings.crosshair);
}

/// Rebuilds the crosshair when the crosshair settings change.
fn rebuild_crosshair(
    settings: Res<Settings>,
    roots: Query<Entity, With<CrosshairRoot>>,
    mut commands: Commands,
) {
    if !settings.is_changed() || roots.is_empty() {
        return;
    }
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    spawn_crosshair_ui(&mut commands, &settings.crosshair);
}

fn despawn_crosshair(mut commands: Commands, roots: Query<Entity, With<CrosshairRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

fn crosshair_color(cfg: &CrosshairCfg) -> Color {
    Color::srgb_u8(cfg.color[0], cfg.color[1], cfg.color[2])
}

/// A rectangular crosshair piece, with a 1 px black outline.
fn bar(left: f32, top: f32, width: f32, height: f32, color: Color) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(left),
            top: Val::Px(top),
            width: Val::Px(width),
            height: Val::Px(height),
            ..default()
        },
        BackgroundColor(color),
        Outline::new(Val::Px(1.0), Val::Px(0.0), Color::BLACK),
    )
}

/// Full-screen centered crosshair with one of the four styles.
pub fn spawn_crosshair_ui(commands: &mut Commands, cfg: &CrosshairCfg) {
    let color = crosshair_color(cfg);
    let size = cfg.size.max(1.0);
    let thickness = cfg.thickness.max(1.0);
    let gap = cfg.gap.max(0.0);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            CrosshairRoot,
        ))
        .with_children(|root| {
            // Zero-sized anchor at the exact screen center.
            root.spawn(Node {
                position_type: PositionType::Absolute,
                width: Val::Px(0.0),
                height: Val::Px(0.0),
                ..default()
            })
            .with_children(|anchor| match cfg.style {
                CrosshairStyle::Dot => {
                    anchor.spawn(bar(
                        -thickness * 0.5,
                        -thickness * 0.5,
                        thickness,
                        thickness,
                        color,
                    ));
                }
                CrosshairStyle::Cross => {
                    spawn_cross_bars(anchor, size, thickness, gap, color, true);
                }
                CrosshairStyle::Circle => {
                    anchor.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(-size * 0.5),
                            top: Val::Px(-size * 0.5),
                            width: Val::Px(size),
                            height: Val::Px(size),
                            border: UiRect::all(Val::Px(thickness)),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BorderColor::all(color),
                        Outline::new(Val::Px(1.0), Val::Px(0.0), Color::BLACK),
                    ));
                }
                CrosshairStyle::T => {
                    spawn_cross_bars(anchor, size, thickness, gap, color, false);
                }
            });
        });
}

fn spawn_cross_bars(
    anchor: &mut ChildSpawnerCommands,
    size: f32,
    thickness: f32,
    gap: f32,
    color: Color,
    with_top: bool,
) {
    let half_thickness = thickness * 0.5;

    if with_top {
        anchor.spawn(bar(
            -half_thickness,
            -(gap + size),
            thickness,
            size,
            color,
        ));
    }
    anchor.spawn(bar(
        -half_thickness,
        gap,
        thickness,
        size,
        color,
    ));
    anchor.spawn(bar(
        -(gap + size),
        -half_thickness,
        size,
        thickness,
        color,
    ));
    anchor.spawn(bar(gap, -half_thickness, size, thickness, color));
}
