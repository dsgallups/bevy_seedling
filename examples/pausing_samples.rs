//! This example demonstrates how to manage sample pausing and playing.

use bevy::{log::LogPlugin, prelude::*};
use bevy_seedling::prelude::*;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            LogPlugin::default(),
            AssetPlugin::default(),
            SeedlingPlugin::default(),
        ))
        .insert_resource(Metronome(Timer::new(
            Duration::from_millis(1500),
            TimerMode::Repeating,
        )))
        .add_systems(Startup, startup)
        .add_systems(Update, toggle_playback)
        .run();
}

// Let's start playing a couple samples.
fn startup(server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(SamplePlayer::new(server.load("caw.ogg")).looping());

    commands.spawn(SamplePlayer::new(server.load("crow_ambience.ogg")).looping());
}

#[derive(Resource)]
struct Metronome(Timer);

fn toggle_playback(
    // With this, we can iterate over _all_ sample players.
    mut players: Query<&mut PlaybackSettings, With<SamplePlayer>>,
    mut metro: ResMut<Metronome>,
    time: Res<Time>,
) {
    let delta = time.delta();
    metro.0.tick(delta);

    if metro.0.just_finished() {
        info!("toggled playback!");

        // The pause and play methods queue up audio events that
        // are sent at the end of the frame.
        for mut player in players.iter_mut() {
            if matches!(*player.playback, PlaybackState::Play { .. }) {
                player.pause();
            } else {
                player.play();
            }
        }
    }
}
