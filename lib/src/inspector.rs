use bevy::prelude::*;
use bevy_inspector_egui::{InspectableRegistry, WorldInspectorPlugin};

use player_state::PlayerState;
use types::Player;

use crate::{
    clock::Clock,
    damage::Health,
    meter::Meter,
    physics::{ConstantVelocity, PlayerVelocity},
};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        let mut registry = app
            .add_plugin(WorldInspectorPlugin::new())
            .insert_resource(InspectableRegistry::default())
            .world
            .get_resource_mut::<InspectableRegistry>()
            .expect("InspectableRegistry not initiated");

        registry.register::<Player>();
        registry.register::<Meter>();
        registry.register::<Health>();
        registry.register::<PlayerState>();
        registry.register::<Clock>();
        registry.register::<PlayerVelocity>();
        registry.register::<ConstantVelocity>();
    }
}
