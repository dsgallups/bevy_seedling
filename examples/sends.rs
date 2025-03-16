//! This example demonstrates how to play a one-shot sample.

use bevy::{log::LogPlugin, prelude::*};
use bevy_seedling::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            LogPlugin::default(),
            AssetPlugin::default(),
            SeedlingPlugin {
                default_pool_size: None,
                ..Default::default()
            },
        ))
        .add_systems(
            Startup,
            |server: Res<AssetServer>, mut commands: Commands| {
                #[derive(NodeLabel, PartialEq, Eq, Hash, Clone, Debug)]
                struct SendLabel;

                commands.spawn((LowPassNode::new(500.0), SendLabel));

                Pool::new(DefaultPool, 16)
                    .spawn(&mut commands)
                    .chain_node(SendNode::new(Volume::Linear(1.0), SendLabel));

                commands.spawn(SamplePlayer::new(server.load("snd_wobbler.wav")));
            },
        )
        .run();
}
