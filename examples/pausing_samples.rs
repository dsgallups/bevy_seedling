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

fn startup(server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn((
        SamplePlayer::new(server.load("caw.ogg")),
        PlaybackSettings::LOOP,
    ));

    commands.spawn((
        SamplePlayer::new(server.load("crow_ambience.ogg")),
        PlaybackSettings::LOOP,
    ));
}

#[derive(Resource)]
struct Metronome(Timer);

fn toggle_playback(
    mut players: Query<&mut PlaybackSettings, With<SamplePlayer>>,
    mut metro: ResMut<Metronome>,
    time: Res<Time>,
) {
    let delta = time.delta();
    metro.0.tick(delta);

    if metro.0.just_finished() {
        bevy_log::info!("toggled playback!");
        for mut player in players.iter_mut() {
            if matches!(*player.playback, PlaybackState::Play { .. }) {
                player.pause();
            } else {
                player.play();
            }
        }
    }
}
