//! This example demonstrates how to route audio to a send.

use bevy::{log::LogPlugin, prelude::*};
use bevy_seedling::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            LogPlugin::default(),
            AssetPlugin::default(),
            SeedlingPlugin::default(),
        ))
        .add_systems(
            Startup,
            |server: Res<AssetServer>, mut commands: Commands| {
                // TODO: use reverb here so you can actually hear the effect
                let send = commands.spawn(LowPassNode::new(500.0)).id();

                commands
                    .spawn(SamplePlayer::new(server.load("caw.ogg")))
                    .effect(SendNode::new(Volume::Linear(1.0), send));
            },
        )
        .run();
}
