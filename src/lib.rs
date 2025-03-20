//! [![crates.io](https://img.shields.io/crates/v/bevy_seedling)](https://crates.io/crates/bevy_seedling)
//! [![docs.rs](https://docs.rs/bevy_seedling/badge.svg)](https://docs.rs/bevy_seedling)
//!
//! A sprouting integration of the [Firewheel](https://github.com/BillyDM/firewheel)
//! audio engine for [Bevy](https://bevyengine.org/).
//!
//! ## Getting started
//!
//! First, you'll need to add the dependency to your `Cargo.toml`.
//! Note that you'll need to disable Bevy's `bevy_audio` feature,
//! meaning you'll need to specify quite a few features
//! manually!
//!
//! ```toml
//! [dependencies]
//! bevy_seedling = "0.3"
//! bevy = { version = "0.15", default-features = false, features = [
//!   "animation",
//!   "bevy_asset",
//!   "bevy_color",
//!   "bevy_core_pipeline",
//!   "bevy_gilrs",
//!   "bevy_gizmos",
//!   "bevy_gltf",
//!   "bevy_mesh_picking_backend",
//!   "bevy_pbr",
//!   "bevy_picking",
//!   "bevy_render",
//!   "bevy_scene",
//!   "bevy_sprite",
//!   "bevy_sprite_picking_backend",
//!   "bevy_state",
//!   "bevy_text",
//!   "bevy_ui",
//!   "bevy_ui_picking_backend",
//!   "bevy_window",
//!   "bevy_winit",
//!   "custom_cursor",
//!   "default_font",
//!   "hdr",
//!   "multi_threaded",
//!   "png",
//!   "smaa_luts",
//!   "sysinfo_plugin",
//!   "tonemapping_luts",
//!   "webgl2",
//!   "x11",
//! ] }
//! ```
//!
//! Then, you'll need to add the [`SeedlingPlugin`] to your app.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_seedling::prelude::*;
//!
//! fn main() {
//!     App::default()
//!         .add_plugins((DefaultPlugins, SeedlingPlugin::default()))
//!         .run();
//! }
//! ```
//!
//! Once you've set it all up, playing sounds is easy!
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_seedling::prelude::*;
//! fn play_sound(mut commands: Commands, server: Res<AssetServer>) {
//!     // Play a sound!
//!     commands.spawn(SamplePlayer::new(server.load("my_sample.wav")));
//!
//!     // Play a sound... with effects :O
//!     commands
//!         .spawn((
//!             SamplePlayer::new(server.load("my_ambience.wav")),
//!             PlaybackSettings::LOOP,
//!         ))
//!         .effect(LowPassNode::new(500.0));
//! }
//! ```
//!
//! [The repository's examples](https://github.com/CorvusPrudens/bevy_seedling/tree/master/examples)
//! should help you get up to speed on common usage patterns.
//!
//! ## Table of contents
//!
//! Below is a structured overview of this crate's documentation,
//! arranged to ease you into `bevy_seedling`'s features.
//!
//! ### Playing samples
//! - [The `SamplePlayer` type][prelude::SamplePlayer]
//! - [Controlling playback][prelude::PlaybackSettings]
//! - [The sample lifecycle][prelude::SamplePlayer#lifecycle]
//! - [Applying effects][prelude::SamplePlayer#applying-effects]
//!
//! ### Sampler pools
//! - [Dynamic pools][pool::dynamic]
//! - [Static pools][prelude::Pool]
//!
//! ### Audio routing
//! - [Connecting nodes][crate::connect::Connect]
//! - [Disconnecting nodes][crate::connect::Disconnect]
//! - [Routing targets][prelude::ConnectTarget]
//! - [Sends][prelude::SendNode]
//!
//! ### Custom nodes
//! - [Creating and registering nodes][prelude::RegisterNode]
//! - [Synchronizing ECS and audio types][prelude::RegisterNode#synchronizing-ecs-and-auto-types]
//!
//! ## Feature flags
//!
//! | Flag | Description | Default feature |
//! | ---  | ----------- | --------------- |
//! | `wav` | Enable WAV format and PCM encoding. | Yes |
//! | `ogg` | Enable Ogg format and Vorbis encoding. | Yes |
//! | `mp3` | Enable mp3 format and encoding. | No |
//! | `mkv` | Enable mkv format. | No |
//! | `adpcm` | Enable adpcm encoding. | No |
//! | `flac` | Enable FLAC format and encoding. | No |
//! | `stream` | Enable CPAL input and output stream nodes. | Yes |
//!
//! ## Frequently asked questions
//!
//! ### Why aren't my mp3 samples making any sound?
//!
//! `bevy_seedling` enables a few formats and encodings by default.
//! If your format isn't included in the [default features][self#feature-flags],
//! you'll need to enable it in your `Cargo.toml`.
//!
//!
//! ```toml
//! [dependencies]
//! bevy_seedling = { version = "0.3.0", features = ["mp3"] }
//! ```
//!
//! ### Why isn't my custom node doing anything?
//!
//! `bevy_seedling` does quite a bit with Firewheel nodes under the hood.
//! To enable this machinery, you need to [register your audio node][prelude::RegisterNode].
//!
//! ```ignore
//! use bevy::prelude::*;
//! use bevy_seedling::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins((DefaultPlugins, SeedlingPlugin::default()))
//!         .register_node::<MyCustomNode>();
//! }
//! ```
//!
//! ### Why are my custom nodes crunchy (underrunning)?
//!
//! If you compile your project without optimizations, your custom audio nodes
//! may perform poorly enough to frequently underrun. You can compensate for
//! this by moving your audio code into a separate crate, selectively applying
//! optimizations.
//!
//! ```toml
//! // Cargo.toml
//! [dependencies]
//! my_custom_nodes = { path = "my_custom_nodes" }
//!
//! [profile.dev.package.my_custom_nodes]
//! opt-level = 3
//! ```
//!
//! ### Why am I getting "`PlaybackSettings`, `Volume`, etc. is ambiguous" errors?
//!
//! `bevy_seedling` re-uses some type names from `bevy_audio`. To avoid ambiguous imports,
//! you'll need to [prevent `bevy_audio` from being compiled][self#getting-started].
//! You may need to update your `Cargo.lock` file to ensure `bevy_audio` isn't included.
//!
//! ## Architecture
//!
//! `bevy_seedling` provides a thin ECS wrapper over `Firewheel`.
//!
//! A `Firewheel` audio node is typically represented in the ECS as
//! an entity with a [`FirewheelNode`][prelude::FirewheelNode] and a component that can generate
//! `Firewheel` events, such as [`VolumeNode`][prelude::VolumeNode].
//!
//! Interactions with the audio engine are buffered.
//! This includes inserting nodes into the audio graph,
//! removing nodes from the graph, making connections
//! between nodes, and sending node events. This provides
//! a few advantages:
//!
//! 1. Audio entities do not need to wait until
//!    they have Firewheel IDs before they can
//!    make connections or generate events.
//! 2. Systems that spawn or interact with
//!    audio entities can be trivially parallelized.
//! 3. Graph-mutating interactions are properly deferred
//!    while the audio graph isn't ready, for example
//!    if it's been temporarily deactiviated.

#![allow(clippy::type_complexity)]
#![expect(clippy::needless_doctest_main)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]

// Naming trick to facilitate straightforward internal macro usage.
extern crate self as bevy_seedling;

use bevy_app::{Last, Plugin, PreStartup};
use bevy_asset::AssetApp;
use bevy_ecs::prelude::*;
use firewheel::{backend::AudioBackend, CpalBackend};

pub mod bpf;
pub mod connect;
pub mod context;
pub mod fixed_vec;
pub mod lpf;
pub mod node;
pub mod pool;
pub mod sample;
pub mod send;
pub mod spatial;
pub mod timeline;

#[cfg(any(feature = "profiling", test))]
pub mod profiling;

pub mod prelude {
    //! All `bevy_seedlings`'s important types and traits.

    pub use crate::bpf::BandPassNode;
    pub use crate::connect::{Connect, ConnectTarget, Disconnect};
    pub use crate::context::AudioContext;
    pub use crate::lpf::LowPassNode;
    pub use crate::node::{
        label::{MainBus, NodeLabel},
        FirewheelNode, RegisterNode,
    };
    pub use crate::pool::{
        dynamic::DynamicPool,
        label::{DefaultPool, PoolLabel},
        Pool, PoolCommands, PoolDespawn,
    };
    pub use crate::sample::{OnComplete, PlaybackSettings, SamplePlayer};
    pub use crate::send::SendNode;
    pub use crate::spatial::{SpatialListener2D, SpatialListener3D};
    pub use crate::SeedlingPlugin;

    pub use firewheel::{
        clock::{ClockSamples, ClockSeconds},
        nodes::{
            sampler::{RepeatMode, SamplerNode},
            spatial_basic::{SpatialBasicConfig, SpatialBasicNode},
            volume::{VolumeNode, VolumeNodeConfig},
            volume_pan::{VolumePanNode, VolumePanNodeConfig},
            StereoToMonoNode,
        },
        FirewheelConfig, Volume,
    };

    #[cfg(feature = "stream")]
    pub use firewheel::nodes::stream::{
        reader::{StreamReaderConfig, StreamReaderNode},
        writer::{StreamWriterConfig, StreamWriterNode},
    };
}

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
#[derive(Debug)]
pub struct SeedlingPlugin<B: AudioBackend = CpalBackend> {
    /// [`firewheel`]'s config, forwarded directly to
    /// the engine.
    ///
    /// [`firewheel`]: firewheel
    pub config: prelude::FirewheelConfig,

    /// The stream settings, forwarded directly to the backend.
    pub stream_config: B::Config,

    /// The number of sampler nodes for the default
    /// sampler pool. If `None` is provided,
    /// the default pool will not be spawned, allowing
    /// you to set it up how you like.
    pub default_pool_size: Option<usize>,

    /// The size range for dynamic pools. Pools
    /// will be spawned with the minimum value,
    /// and will grow depending on demand to the
    /// maximum size. Setting this field to `None`
    /// will disabled dynamic pools entirely.
    pub dynamic_pool_range: Option<core::ops::RangeInclusive<usize>>,
}

impl Default for SeedlingPlugin<CpalBackend> {
    fn default() -> Self {
        SeedlingPlugin::<CpalBackend>::new()
    }
}

impl<B: AudioBackend> SeedlingPlugin<B>
where
    B::Config: Default,
{
    /// Create a new default [`SeedlingPlugin`].
    pub fn new() -> Self {
        Self {
            config: Default::default(),
            stream_config: Default::default(),
            default_pool_size: Some(24),
            dynamic_pool_range: Some(4..=16),
        }
    }
}

impl<B: AudioBackend> Plugin for SeedlingPlugin<B>
where
    B: 'static,
    B::Config: Clone + Send + Sync + 'static,
    B::StreamError: Send + Sync + 'static,
{
    fn build(&self, app: &mut bevy_app::App) {
        use prelude::*;

        let mut context = AudioContext::new::<B>(self.config, self.stream_config.clone());
        let sample_rate = context.with(|ctx| ctx.stream_info().unwrap().sample_rate);
        let sample_pool_size = self.default_pool_size;

        app.insert_resource(context)
            .init_resource::<connect::NodeMap>()
            .init_resource::<node::PendingRemovals>()
            .insert_resource(pool::dynamic::DynamicPoolRange(
                self.dynamic_pool_range.clone(),
            ))
            .init_asset::<sample::Sample>()
            .register_asset_loader(sample::SampleLoader { sample_rate })
            .register_node::<lpf::LowPassNode>()
            .register_node::<bpf::BandPassNode>()
            .register_node::<send::SendNode>()
            .register_node::<VolumeNode>()
            .register_node::<VolumePanNode>()
            .register_node::<SpatialBasicNode>()
            .register_simple_node::<StereoToMonoNode>()
            .register_simple_node::<SamplerNode>();

        #[cfg(feature = "stream")]
        app.register_simple_node::<StreamReaderNode>()
            .register_simple_node::<StreamWriterNode>();

        app.configure_sets(
            Last,
            (
                SeedlingSystems::Connect.after(SeedlingSystems::Acquire),
                SeedlingSystems::Queue.after(SeedlingSystems::Acquire),
                SeedlingSystems::Flush
                    .after(SeedlingSystems::Connect)
                    .after(SeedlingSystems::Queue),
            ),
        )
        .add_systems(
            Last,
            (
                (
                    spatial::update_2d_emitters,
                    spatial::update_3d_emitters,
                    send::connect_sends,
                    send::update_remote_sends,
                )
                    .before(SeedlingSystems::Acquire),
                connect::auto_connect
                    .before(SeedlingSystems::Connect)
                    .after(SeedlingSystems::Acquire),
                (
                    connect::process_connections,
                    connect::process_disconnections,
                )
                    .in_set(SeedlingSystems::Connect),
                (
                    node::process_removals,
                    node::flush_events,
                    context::update_context,
                )
                    .chain()
                    .in_set(SeedlingSystems::Flush),
            ),
        )
        .add_systems(
            PreStartup,
            (
                node::label::insert_main_bus,
                move |mut commands: Commands| {
                    if let Some(size) = sample_pool_size {
                        Pool::new(DefaultPool, size).spawn(&mut commands);
                    }
                },
            ),
        );

        app.add_plugins(pool::SamplePoolPlugin);
    }
}
