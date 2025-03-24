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
                // Let's consider this our effects send.
                let send = commands
                    .spawn(FreeverbNode {
                        room_size: 0.9,
                        width: 0.75,
                        damping: 0.5,
                    })
                    .id();

                // We can insert a send anywhere, including directly on a
                // sample player.
                commands
                    .spawn(SamplePlayer::new(server.load("caw.ogg")))
                    // Here we send some of the signal to our effects.
                    .effect(SendNode::new(Volume::Linear(0.75), send));
            },
        )
        .run();
}
