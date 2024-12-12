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
                info!("Starting up");

                commands.spawn(SamplePlayer::new(server.load("snd_wobbler.wav")));
            },
        )
        .run();
}
