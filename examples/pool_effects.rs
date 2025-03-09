//! This example demonstrates how to spawn a sample pool with custom effects.

use bevy::{log::LogPlugin, prelude::*};
use bevy_seedling::{
    lpf::LowPassNode, sample::SamplePlayer, PoolLabel, SeedlingPlugin, SpawnPool, VolumeNode,
};

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            LogPlugin::default(),
            AssetPlugin::default(),
            SeedlingPlugin::default(),
        ))
        .add_systems(Startup, startup)
        .run();
}

fn startup(server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(SamplePlayer::new(server.load("snd_wobbler.wav")));

    // normal approach
    #[derive(PoolLabel)]
    struct MyPool;

    commands
        .spawn_pool(MyPool, 4)
        .chain(LowPassNode { frequency: 1000.0 })
        .chain(VolumeNode {
            normalized_volume: 1.0,
        });

    commands.spawn((
        MyPool,
        SamplePlayer::new(server.load("snd_wobbler.wav")),
        LowPassNode { frequency: 1000.0 },
        VolumeNode {
            normalized_volume: 1.0,
        },
    ));

    // // insane ergo
    // commands.spawn((
    //     SamplePlayer::new(server.load("snd_wobbler.wav")),
    //     LowPassNode { frequency: 1000.0 },
    //     VolumeNode {
    //         normalized_volume: 1.0,
    //     },
    // ));
}
