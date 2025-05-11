//! This example demonstrates how to set up a
//! custom sample pool, a custom bus, and the routing in-between.

use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use bevy_seedling::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            LogPlugin {
                level: Level::DEBUG,
                ..Default::default()
            },
            AssetPlugin::default(),
            SeedlingPlugin::default(),
        ))
        .add_systems(
            Startup,
            |server: Res<AssetServer>, mut commands: Commands| {
                commands.spawn((
                    SamplePlayer::new(server.load("caw.ogg")),
                    PlaybackSettings::LOOP,
                    sample_effects![
                        LowPassNode::default(),
                        VolumePanNode {
                            pan: 0.5,
                            ..Default::default()
                        }
                    ],
                ));

                commands.spawn((
                    SamplePlayer::new(server.load("caw.ogg")),
                    PlaybackSettings::LOOP,
                    sample_effects![
                        LowPassNode { frequency: 250. },
                        VolumePanNode {
                            pan: -0.5,
                            ..Default::default()
                        }
                    ],
                ));
            },
        )
        .run();
}
