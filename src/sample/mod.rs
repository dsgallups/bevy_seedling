//! Audio sample components.

use crate::node::ExcludeNode;
use crate::prelude::Volume;
use bevy_asset::Handle;
use bevy_ecs::{component::ComponentId, prelude::*, world::DeferredWorld};
use firewheel::nodes::sampler::RepeatMode;

mod assets;

pub use assets::{Sample, SampleLoader, SampleLoaderError};

/// A component that queues sample playback.
///
/// ## Playing sounds
///
/// Playing a sound is very simple!
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_seedling::prelude::*;
/// fn play_sound(mut commands: Commands, server: Res<AssetServer>) {
///     commands.spawn(SamplePlayer::new(server.load("my_sample.wav")));
/// }
/// ```
///
/// This queues up playback in a [*sampler pool*][crate::prelude::Pool].
/// Without any effects applied, samples are played in the
/// [`DefaultPool`][crate::prelude::DefaultPool].
///
/// To control playback, such as enabling looping, you can
/// also provide a [`PlaybackSettings`] component.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_seedling::prelude::*;
/// fn play_looping_sound(mut commands: Commands, server: Res<AssetServer>) {
///     commands.spawn((
///         SamplePlayer::new(server.load("my_sample.wav")),
///         PlaybackSettings::LOOP,
///     ));
/// }
/// ```
///
/// ## Lifecycle
///
/// By default, entities with a [`SamplePlayer`] component are despawned when
/// playback completes. If you insert [`SamplePlayer`] components on gameplay entities
/// such as the player or enemies, you'll probably want to set [`PlaybackSettings::on_complete`]
/// to [`OnComplete::Remove`] or even [`OnComplete::Preserve`].
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_seedling::prelude::*;
/// #[derive(Component)]
/// struct Player;
///
/// #[derive(Event)]
/// struct SoundEvent;
///
/// fn play_sound_on_player(
///     q: Query<Entity, With<Player>>,
///     mut sound_events: EventReader<SoundEvent>,
///     server: Res<AssetServer>,
///     mut commands: Commands,
/// ) {
///     let player = q.single();
///
///     for _ in sound_events.read() {
///         commands.entity(player).insert((
///             SamplePlayer::new(server.load("my_sample.wav")),
///             PlaybackSettings {
///                 on_complete: OnComplete::Remove,
///                 ..Default::default()
///             },
///         ));
///     }
/// }
/// ```
///
/// ## Applying effects
///
/// Effects can be applied directly to a sample entity with the
/// [`DynamicPool`][crate::prelude::DynamicPool] trait.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_seedling::prelude::*;
/// fn play_with_effects(mut commands: Commands, server: Res<AssetServer>) {
///     commands
///         .spawn(SamplePlayer::new(server.load("my_sample.wav")))
///         .effect(SpatialBasicNode::default())
///         .effect(LowPassNode::new(500.0));
/// }
/// ```
///
/// This connects a spatial and low-pass node in series with the sample player in the above example.
/// Effects are arranged in the order of `effect` calls, so the output of the spatial node is
/// connected to the input of the low-pass node.
///
/// When you apply effects to a sample player, the node components are added directly to the
/// entity as [*remote nodes*][crate::node::ExcludeNode]. That allows you to modulate node
/// parameters directly on your sample player entity.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_seedling::prelude::*;
/// fn modulate_remote_nodes(mut q: Query<&mut LowPassNode, With<SamplePlayer>>) {
///     for mut low_pass_params in q.iter_mut() {
///         low_pass_params.frequency.set(1000.0);
///     }
/// }
/// ```
///
/// Applying effects directly to a [`SamplePlayer`] is simple, but it
/// [has some tradeoffs][crate::pool::dynamic#when-to-use-dynamic-pools], so you may
/// find yourself gravitating towards manually defined [`Pool`][crate::prelude::Pool]s as your
/// requirements grow.
#[derive(Debug, Component, Clone)]
#[require(PlaybackSettings, ExcludeNode)]
#[component(on_insert = on_insert_sample)]
pub struct SamplePlayer(pub(crate) Handle<Sample>);

fn on_insert_sample(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
    world.commands().entity(entity).insert(QueuedSample);
}

impl SamplePlayer {
    /// Construct a new [`SamplePlayer`].
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_seedling::prelude::*;
    /// fn play_sound(mut commands: Commands, server: Res<AssetServer>) {
    ///     commands.spawn(SamplePlayer::new(server.load("my_sample.wav")));
    /// }
    /// ```
    pub fn new(handle: Handle<Sample>) -> Self {
        Self(handle)
    }
}

/// Controls the playback settings of a [`SamplePlayer`].
#[derive(Debug, Component, Clone, Default)]
pub struct PlaybackSettings {
    /// Sets the sample's [`RepeatMode`].
    pub repeat_mode: RepeatMode,
    /// Determines this sample's behavior on playback completion.
    pub on_complete: OnComplete,
    /// Sets the volume of the sample.
    pub volume: Volume,
}

impl PlaybackSettings {
    /// Play the audio source once, despawning
    /// this entity when complete or interrupted.
    pub const ONCE: Self = Self {
        repeat_mode: RepeatMode::PlayOnce,
        volume: Volume::Linear(1.0),
        on_complete: OnComplete::Despawn,
    };

    /// Repeatedly loop the audio source until
    /// this entity is despawned.
    pub const LOOP: Self = Self {
        repeat_mode: RepeatMode::RepeatEndlessly,
        volume: Volume::Linear(1.0),
        on_complete: OnComplete::Despawn,
    };

    /// Play the sample once, removing the audio-related components on completion.
    pub const REMOVE: Self = Self {
        repeat_mode: RepeatMode::PlayOnce,
        volume: Volume::Linear(1.0),
        on_complete: OnComplete::Remove,
    };

    /// Play the sample once, preserving the components and entity on completion.
    pub const PRESERVE: Self = Self {
        repeat_mode: RepeatMode::PlayOnce,
        volume: Volume::Linear(1.0),
        on_complete: OnComplete::Preserve,
    };
}

/// Determines what happens when a sample completes plaback.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum OnComplete {
    /// Preserve the entity and components, leaving them untouched.
    Preserve,
    /// Remove the [`SamplePlayer`] and related components.
    Remove,
    /// Despawn the [`SamplePlayer`] entity.
    #[default]
    Despawn,
}

/// A marker struct for entities that are waiting
/// for asset loading and playback assignment.
#[derive(Debug, Component, Default)]
#[component(storage = "SparseSet")]
pub struct QueuedSample;
