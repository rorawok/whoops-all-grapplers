use bevy::prelude::*;

use super::{Colors, Fonts, Models, Sprites};

pub fn colors(mut commands: Commands) {
    commands.insert_resource(Colors {
        health: Color::rgb(0.9, 0.0, 0.0),
        meter: Color::rgb(0.04, 0.5, 0.55),
        charge_default: Color::rgb(0.05, 0.4, 0.55),
        charge_full: Color::rgb(0.9, 0.1, 0.3),
        hitbox: Color::rgb(1.0, 0.0, 0.0),
        hurtbox: Color::rgb(0.0, 1.0, 0.0),
        collision_box: Color::rgba(0.0, 0.0, 1.0, 0.75),
        text: Color::WHITE,
    })
}

pub fn fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    let basic = asset_server.load("FiraSans-Bold.ttf");

    commands.insert_resource(Fonts { basic })
}

pub fn sprites(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Sprites {
        background_image: asset_server.load("CPT-2018-Stage.png"),
    });
}

pub fn models(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Models {
        ryan: asset_server.load("dummy-character.glb"),
    });
}
