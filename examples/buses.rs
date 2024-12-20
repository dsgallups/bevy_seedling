//! This example demonstrates how to set up a
//! custom bus and route nodes to it.

use bevy::{log::LogPlugin, prelude::*};
use bevy_seedling::{
    firewheel::{basic_nodes::VolumeNode, clock::ClockSeconds},
    label::InternedLabel,
    lpf::LowPassNode,
    sample::SamplePlayer,
    AudioContext, ConnectNode, MainBus, NodeLabel, SeedlingPlugin,
};

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
                // Any arbitrary effects chain can go here, but
                // let's just insert a low-pass filter.
                //
                // This node is implicitly connected to the `MainBus`.
                let effects = commands.spawn(LowPassNode::new(10000.)).id();

                // Here we create a volume node that acts as the entry to
                // our effects bus and we connect it to the effects.
                //
                // When we spawn it with the `EffectsBus` label, any node
                // can use this type to connect to this node anywhere in
                // the code.
                commands
                    .spawn((VolumeNode::new(1.), EffectsBus))
                    .connect(effects);

                // Since we're here, we might as well trigger a one-shot sample
                // and send it through the effects chain.
                commands
                    .spawn(SamplePlayer::new(server.load("snd_wobbler.wav")))
                    .connect(EffectsBus);
            },
        )
        .add_systems(
            PostStartup,
            // Here we apply some modulation to the frequency of the low-pass filter.
            |q: Single<&mut LowPassNode>, mut context: ResMut<AudioContext>| {
                let now = context.now();
                let mut node = q.into_inner();

                node.frequency
                    .push_curve(
                        0.,
                        now,
                        now + ClockSeconds(0.5),
                        EaseFunction::ExponentialOut,
                    )
                    .unwrap();

                node.frequency
                    .push_curve(
                        10000.,
                        now + ClockSeconds(0.5),
                        now + ClockSeconds(2.0),
                        EaseFunction::ExponentialOut,
                    )
                    .unwrap();
            },
        )
        .run();
}
