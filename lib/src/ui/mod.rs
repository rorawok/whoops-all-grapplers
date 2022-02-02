use bevy::prelude::*;
use time::{GameState, RoundTimer, ROUND_TIME};
use types::Player;

use crate::{
    assets::{Colors, Fonts, Sprites},
    labels::StartupStageLabel,
};

mod bars;
mod round_text;

use bars::{HealthBar, MeterBar};
use round_text::RoundText;

// Top bars
const TOP_CONTAINER_TOP_PAD: f32 = 0.0;
const TOP_CONTAINER_SIDE_PAD: f32 = 5.0;
const TOP_CONTAINER_WIDTH: f32 = 100.0 - 2.0 * TOP_CONTAINER_SIDE_PAD;
const TOP_CONTAINER_HEIGHT: f32 = 10.0;

const TIMER_WIDTH: f32 = 10.0;
const TIMER_TOP_PADDING: f32 = 2.0;
const HEALTH_BAR_WIDTH: f32 = (100.0 - TIMER_WIDTH) / 2.0; // Relative to wrapper
const HEALTH_BAR_HEIGHT: f32 = 50.0; // Relative to wrapper

// Bottom bars
const BOTTOM_CONTAINER_BOTTOM_PAD: f32 = 3.0;
const BOTTOM_CONTAINER_SIDE_PAD: f32 = 3.0;
const BOTTOM_CONTAINER_WIDTH: f32 = 100.0 - 2.0 * BOTTOM_CONTAINER_SIDE_PAD;
const BOTTOM_CONTAINER_HEIGHT: f32 = 3.0;
const METER_BAR_WIDTH: f32 = 30.0; // Relative to wrapper
const METER_BAR_HEIGHT: f32 = 100.0; // Relative to wrapper

const BACKGROUND_POSITION: (f32, f32, f32) = (0.0, 2.0, -0.09);
const BACKGROUND_SCALE: (f32, f32, f32) = (0.008, 0.008, 1.0);

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStageLabel::UI, setup_ui)
            .add_system(bars::update)
            .add_system_set(
                SystemSet::on_enter(GameState::Combat).with_system(round_text::round_start),
            )
            .add_system_set(
                SystemSet::on_update(GameState::PostRound).with_system(round_text::round_over),
            )
            .add_startup_system(add_stage);
    }
}

fn add_stage(mut commands: Commands, sprites: Res<Sprites>) {
    commands.spawn_bundle(SpriteBundle {
        texture: sprites.background_image.clone(),
        transform: Transform {
            translation: BACKGROUND_POSITION.into(),
            scale: BACKGROUND_SCALE.into(),
            ..Default::default()
        },

        ..Default::default()
    });
}

fn setup_ui(mut commands: Commands, colors: Res<Colors>, fonts: Res<Fonts>) {
    setup_top_bars(&mut commands, &colors, &fonts);
    setup_bottom_bars(&mut commands, &colors);
    setup_round_info_text(&mut commands, &colors, &fonts);
}

fn setup_top_bars(commands: &mut Commands, colors: &Colors, fonts: &Fonts) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                size: Size::new(
                    Val::Percent(TOP_CONTAINER_WIDTH),
                    Val::Percent(TOP_CONTAINER_HEIGHT),
                ),
                position: Rect {
                    top: Val::Percent(TOP_CONTAINER_TOP_PAD),
                    left: Val::Percent(TOP_CONTAINER_SIDE_PAD),
                    ..Default::default()
                },
                ..Default::default()
            },
            color: colors.transparent.into(),
            ..Default::default()
        })
        .with_children(|top_bar_wrapper| {
            top_bar_wrapper
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(
                            Val::Percent(HEALTH_BAR_WIDTH),
                            Val::Percent(HEALTH_BAR_HEIGHT),
                        ),
                        ..Default::default()
                    },
                    color: colors.health.into(),
                    ..Default::default()
                })
                .insert(HealthBar(Player::One));
            top_bar_wrapper
                .spawn_bundle(NodeBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        size: Size::new(Val::Percent(TIMER_WIDTH), Val::Percent(100.0)),
                        position: Rect {
                            top: Val::Percent(TIMER_TOP_PADDING),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    color: colors.transparent.into(),
                    ..Default::default()
                })
                .with_children(|timer_wrapper| {
                    timer_wrapper
                        .spawn_bundle(TextBundle {
                            text: Text::with_section(
                                ROUND_TIME.round().to_string(),
                                TextStyle {
                                    font: fonts.basic.clone(),
                                    font_size: 100.0,
                                    color: Color::WHITE,
                                },
                                TextAlignment {
                                    horizontal: HorizontalAlign::Center,
                                    vertical: VerticalAlign::Center,
                                },
                            ),
                            ..Default::default()
                        })
                        .insert(RoundTimer);
                });
            top_bar_wrapper
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(
                            Val::Percent(HEALTH_BAR_WIDTH),
                            Val::Percent(HEALTH_BAR_HEIGHT),
                        ),
                        ..Default::default()
                    },
                    color: colors.health.into(),
                    ..Default::default()
                })
                .insert(HealthBar(Player::Two));
        });
}

fn setup_bottom_bars(commands: &mut Commands, colors: &Colors) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::SpaceBetween,
                size: Size::new(
                    Val::Percent(BOTTOM_CONTAINER_WIDTH),
                    Val::Percent(BOTTOM_CONTAINER_HEIGHT),
                ),
                position: Rect {
                    bottom: Val::Percent(BOTTOM_CONTAINER_BOTTOM_PAD),
                    left: Val::Percent(BOTTOM_CONTAINER_SIDE_PAD),
                    ..Default::default()
                },
                ..Default::default()
            },
            color: colors.transparent.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(
                            Val::Percent(METER_BAR_WIDTH),
                            Val::Percent(METER_BAR_HEIGHT),
                        ),
                        ..Default::default()
                    },
                    color: colors.meter.into(),
                    ..Default::default()
                })
                .insert(MeterBar(Player::One));
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(
                            Val::Percent(METER_BAR_WIDTH),
                            Val::Percent(METER_BAR_HEIGHT),
                        ),
                        ..Default::default()
                    },
                    color: colors.meter.into(),
                    ..Default::default()
                })
                .insert(MeterBar(Player::Two));
        });
}

fn setup_round_info_text(commands: &mut Commands, colors: &Colors, fonts: &Fonts) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                size: Size::new(Val::Percent(100.0), Val::Percent(10.0)),
                position: Rect {
                    top: Val::Percent(40.0),
                    left: Val::Px(0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            color: colors.transparent.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text::with_section(
                        "New round",
                        TextStyle {
                            font: fonts.basic.clone(),
                            font_size: 100.0,
                            color: Color::WHITE,
                        },
                        TextAlignment {
                            horizontal: HorizontalAlign::Center,
                            ..Default::default()
                        },
                    ),
                    ..Default::default()
                })
                .insert(RoundText);
        });
}
