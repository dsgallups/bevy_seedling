use bevy::{log::LogPlugin, prelude::*};
use bevy_seedling::{
    node::Params, sample::SamplePlayer, volume::Volume, AudioContext, ConnectNode, SeedlingPlugin,
};
use firewheel::{basic_nodes::VolumeParams, clock::ClockSeconds};

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
                let volume = commands.spawn(Volume::new(0.25)).id();
                commands
                    .spawn(SamplePlayer::new(server.load("snd_wobbler.wav")))
                    .connect(volume);
            },
        )
        .add_systems(
            PostStartup,
            |mut q: Query<&mut Params<VolumeParams>>, mut context: ResMut<AudioContext>| {
                let now = context.now();
                for mut volume in q.iter_mut() {
                    volume
                        .gain
                        .push_curve(
                            0.,
                            now,
                            now + ClockSeconds(1.),
                            EaseFunction::ExponentialOut,
                        )
                        .unwrap();
                }
            },
        )
        .run();
}
