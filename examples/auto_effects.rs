//! This example demonstrates how to play a one-shot sample with effects.

use bevy::{log::LogPlugin, prelude::*};
use bevy_seedling::{pool::auto::AutoPool, prelude::*};

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
                // Spawning a `SamplePlayer` component will play a sample
                // once as soon as it's loaded.
                commands
                    .spawn(SamplePlayer::new(server.load("snd_wobbler.wav")))
                    .effect(BandPassNode::default())
                    .effect(LowPassNode::default());
            },
        )
        .run();
}
