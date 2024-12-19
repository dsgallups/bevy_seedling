use bevy::{log::LogPlugin, prelude::*};
use bevy_seedling::{
    label::InternedLabel, sample::SamplePlayer, AudioContext, ConnectNode, MainBus, NodeLabel,
    SeedlingPlugin,
};
use firewheel::{basic_nodes::VolumeNode, clock::ClockSeconds};

#[derive(NodeLabel, PartialEq, Eq, Debug, Hash, Clone)]
struct EffectsBus;

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
                commands.spawn((VolumeNode::new(1.), InternedLabel::new(EffectsBus)));
            },
        )
        .add_systems(
            PostStartup,
            |q: Single<&mut VolumeNode, With<MainBus>>, mut context: ResMut<AudioContext>| {
                let now = context.now();
                let mut volume = q.into_inner();

                volume
                    .0
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
