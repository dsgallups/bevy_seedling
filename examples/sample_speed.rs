//! This example demonstrates how to adjust a sample's speed during playback.

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
                commands.spawn((
                    SamplePlayer::new(server.load("selfless_courage.ogg")),
                    PlaybackSettings {
                        volume: Volume::Decibels(-6.0),
                        ..PlaybackSettings::LOOP
                    },
                ));
            },
        )
        .add_systems(Update, modulate_speed)
        .run();
}

// The key component is `PlaybackParams`. It's a set of parameters
// that can be changed during playback, unlike `PlaybackSettings` which
// only takes effect once at the beginning of playback.
fn modulate_speed(player: Single<&mut PlaybackParams>, mut angle: Local<f32>, time: Res<Time>) {
    let mut params = player.into_inner();

    params.speed = angle.sin() as f64 * 0.05 + 1.0;

    let period = 10.0;
    *angle += time.delta_secs() * core::f32::consts::TAU / period;
}
