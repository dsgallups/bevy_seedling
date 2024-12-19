//! A sprouting implementation of the [Firewheel](https://github.com/BillyDM/firewheel) audio engine for Bevy.

#![allow(clippy::type_complexity)]

extern crate self as bevy_seedling;

use bevy_app::{Last, Plugin, Startup};
use bevy_asset::AssetApp;
use bevy_ecs::prelude::*;
use firewheel::FirewheelConfig;

pub mod context;
pub mod label;
pub mod node;
pub mod sample;
pub mod volume;

pub use context::AudioContext;
pub use label::{MainBus, NodeLabel};
use node::RegisterNode;
pub use node::{ConnectNode, ConnectTarget, Node};

/// Node label derive macro.
///
/// Node labels provide a convenient way to manage
/// connections with frequently used nodes.
///
/// ```
/// # use crate::NodeLabel;
/// #[derive(NodeLabel, Debug, Clone, PartialEq, Eq, Hash)]
/// struct EffectsChain;
///
/// fn system(server: Res<AssetServer>, mut commands: Commands) {
///     commands.spawn((Volume::new(0.25), InternedLabel::new(EffectsChain)));
///
///     // Now, any node can simply use `EffectsChain`
///     // as a connection target.
///     commands
///         .spawn(SamplePlayer::new(server.load("sound.wav")))
///         .connect(EffectsChain);
/// }
/// ```
pub use seedling_macros::NodeLabel;

/// Sets for all `bevy_seedling` systems.
///
/// These are all inserted into the [`Last`] schedule.
///
/// [`Last`]: bevy_app::Last
#[derive(Debug, SystemSet, PartialEq, Eq, Hash, Clone)]
pub enum SeedlingSystems {
    /// Entities without audio nodes acquire them from the audio context.
    Acquire,
    /// Pending connections are made.
    Connect,
    /// Queue audio engine events.
    ///
    /// While it's not strictly necessary to separate this
    /// set from [`SeedlingSystems::Connect`], it's a nice
    /// semantic divide.
    Queue,
    /// The audio context is updated and flushed.
    Flush,
}

/// `bevy_seedling`'s top-level plugin.
///
/// This spawns the audio task in addition
/// to inserting `bevy_seedling`'s systems
/// and resources.
#[derive(Default)]
pub struct SeedlingPlugin {
    /// [`firewheel`]'s config, forwarded directly to
    /// the engine.
    ///
    /// [`firewheel`]: firewheel
    pub settings: FirewheelConfig,
}

impl Plugin for SeedlingPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        let mut context = AudioContext::new(self.settings);
        let sample_rate = context.with(|ctx| ctx.stream_info().unwrap().sample_rate);

        app.insert_resource(context)
            .init_resource::<node::ParamSystems>()
            .init_resource::<node::NodeMap>()
            .init_resource::<node::PendingRemovals>()
            .init_asset::<sample::Sample>()
            .register_asset_loader(sample::SampleLoader { sample_rate })
            .register_node::<sample::SamplePlayer>()
            .register_node::<volume::Volume>()
            .configure_sets(
                Last,
                (
                    SeedlingSystems::Connect.after(SeedlingSystems::Acquire),
                    SeedlingSystems::Queue.after(SeedlingSystems::Acquire),
                    SeedlingSystems::Flush
                        .after(SeedlingSystems::Connect)
                        .after(SeedlingSystems::Queue),
                ),
            )
            .add_systems(Startup, label::insert_main_bus)
            .add_systems(
                Last,
                (
                    sample::on_add.in_set(SeedlingSystems::Acquire),
                    node::auto_connect
                        .before(SeedlingSystems::Connect)
                        .after(SeedlingSystems::Acquire),
                    node::process_connections.in_set(SeedlingSystems::Connect),
                    sample::trigger_pending_samples.in_set(SeedlingSystems::Queue),
                    (
                        node::process_removals,
                        node::flush_events,
                        context::update_context,
                    )
                        .chain()
                        .in_set(SeedlingSystems::Flush),
                ),
            );
    }
}
