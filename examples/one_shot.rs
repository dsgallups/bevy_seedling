//! This example demonstrates how to play a one-shot sample.

use bevy::{log::LogPlugin, prelude::*};
use bevy_seedling::{sample::SamplePlayer, SeedlingPlugin};

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
                // Spawning a `SamplePlayer` node will play a sample
                // once as soon as it's loaded.
                //
                // This node is implicitly connected to the `MainBus`.
                commands.spawn(SamplePlayer::new(server.load("snd_wobbler.wav")));
            },
        )
        .run();
}
