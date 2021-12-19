use bevy::prelude::*;
use types::Player;

use crate::{
    assets::{Colors, Fonts, Sprites},
    clock,
    game_flow::GameState,
    labels::StartupStageLabel,
};

mod bars;
mod round_text;

use bars::{HealthBar, MeterBar};
use round_text::RoundText;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system_to_stage(StartupStageLabel::UI, setup_ui.system())
            .add_system(bars::update.system())
            .add_system_set(
                SystemSet::on_enter(GameState::Combat)
                    .with_system(round_text::round_start.system()),
            )
            .add_system_set(
                SystemSet::on_update(GameState::PostRound)
                    .with_system(round_text::round_over.system()),
            )
            .add_startup_system(add_stage.system());
    }
}

fn add_stage(mut commands: Commands, sprites: Res<Sprites>, mut meshes: ResMut<Assets<Mesh>>) {
    // TODO: This could probably be made better with some other mechanism.

    let uvs = vec![[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];

    let mut mesh = Mesh::from(shape::Quad::new(Vec2::new(1.0, 1.0)));
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    commands.spawn_bundle(PbrBundle {
        material: sprites.background_image.clone(),
        mesh: meshes.add(mesh),
        transform: Transform {
            translation: constants::BACKGROUND_POSITION.into(),
            scale: constants::BACKGROUND_SCALE.into(),
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
                    Val::Percent(constants::TOP_CONTAINER_WIDTH),
                    Val::Percent(constants::TOP_CONTAINER_HEIGHT),
                ),
                position: Rect {
                    top: Val::Percent(constants::TOP_CONTAINER_TOP_PAD),
                    left: Val::Percent(constants::TOP_CONTAINER_SIDE_PAD),
                    ..Default::default()
                },
                ..Default::default()
            },
            material: colors.transparent.clone(),
            ..Default::default()
        })
        .with_children(|top_bar_wrapper| {
            top_bar_wrapper
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(
                            Val::Percent(constants::HEALTH_BAR_WIDTH),
                            Val::Percent(constants::HEALTH_BAR_HEIGHT),
                        ),
                        ..Default::default()
                    },
                    material: colors.health.clone(),
                    ..Default::default()
                })
                .insert(HealthBar(Player::One));
            top_bar_wrapper
                .spawn_bundle(NodeBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        size: Size::new(Val::Percent(constants::TIMER_WIDTH), Val::Percent(100.0)),
                        position: Rect {
                            top: Val::Percent(constants::TIMER_TOP_PADDING),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    material: colors.transparent.clone(),
                    ..Default::default()
                })
                .with_children(|timer_wrapper| {
                    timer_wrapper
                        .spawn_bundle(TextBundle {
                            text: Text::with_section(
                                constants::ROUND_TIME.round().to_string(),
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
                        .insert(clock::Timer);
                });
            top_bar_wrapper
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(
                            Val::Percent(constants::HEALTH_BAR_WIDTH),
                            Val::Percent(constants::HEALTH_BAR_HEIGHT),
                        ),
                        ..Default::default()
                    },
                    material: colors.health.clone(),
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
                    Val::Percent(constants::BOTTOM_CONTAINER_WIDTH),
                    Val::Percent(constants::BOTTOM_CONTAINER_HEIGHT),
                ),
                position: Rect {
                    bottom: Val::Percent(constants::BOTTOM_CONTAINER_BOTTOM_PAD),
                    left: Val::Percent(constants::BOTTOM_CONTAINER_SIDE_PAD),
                    ..Default::default()
                },
                ..Default::default()
            },
            material: colors.transparent.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(
                            Val::Percent(constants::METER_BAR_WIDTH),
                            Val::Percent(constants::METER_BAR_HEIGHT),
                        ),
                        ..Default::default()
                    },
                    material: colors.meter.clone(),
                    ..Default::default()
                })
                .insert(MeterBar(Player::One));
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(
                            Val::Percent(constants::METER_BAR_WIDTH),
                            Val::Percent(constants::METER_BAR_HEIGHT),
                        ),
                        ..Default::default()
                    },
                    material: colors.meter.clone(),
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
            material: colors.transparent.clone(),
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
