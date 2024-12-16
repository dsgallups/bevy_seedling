use bevy::{log::LogPlugin, prelude::*};
use bevy_seedling::{
    node::Params, sample::SamplePlayer, volume::Volume, AudioContext, ConnectNode, MainBus,
    SeedlingPlugin,
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
                commands.spawn(SamplePlayer::new(server.load("snd_wobbler.wav")));
            },
        )
        .add_systems(
            PostStartup,
            |q: Single<&mut Params<VolumeParams>, With<MainBus>>,
             mut context: ResMut<AudioContext>| {
                let now = context.now();
                let mut volume = q.into_inner();

                volume
                    .gain
                    .push_curve(
                        0.,
                        now,
                        now + ClockSeconds(1.),
                        EaseFunction::ExponentialOut,
                    )
                    .unwrap();
            },
        )
        .run();
}
