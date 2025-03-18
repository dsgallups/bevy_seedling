//! This example demonstrates how to play a one-shot sample.

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
                #[derive(NodeLabel, PartialEq, Eq, Hash, Clone, Debug)]
                struct EffectsSend;

                // TODO: use reverb here so you can actually hear the effect
                commands.spawn((LowPassNode::new(500.0), EffectsSend));

                commands
                    .spawn(SamplePlayer::new(server.load("caw.ogg")))
                    .effect(SendNode::new(Volume::Linear(1.0), EffectsSend));
            },
        )
        .run();
}
