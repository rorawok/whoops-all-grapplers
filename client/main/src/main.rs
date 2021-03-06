// use bevy::{diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}};
// use bevy::ecs::schedule::ReportExecutionOrderAmbiguities;
use bevy::prelude::*;
use whoops_all_grapplers_lib::WAGLib;

fn main() {
    // Happens roughly in order, so add stages, click and assets before using them
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WAGLib)
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .insert_resource(ReportExecutionOrderAmbiguities)
        .run();
}
