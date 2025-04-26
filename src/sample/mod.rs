//! Audio sample components.

use crate::node::ExcludeNode;
use crate::prelude::Volume;
use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};
use firewheel::{
    diff::Notify,
    nodes::sampler::{PlaybackState, Playhead, RepeatMode, SamplerState},
};

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
/// [`PoolBuilder`][crate::prelude::PoolBuilder] trait.
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
/// In the above example, we connect a spatial and low-pass node in series with the sample player.
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
#[require(PlaybackSettings, PlaybackParams, ExcludeNode)]
#[component(on_insert = on_insert_sample)]
pub struct SamplePlayer {
    pub(crate) sample: Handle<Sample>,
    player: Option<Player>,
}

fn on_insert_sample(mut world: DeferredWorld, context: HookContext) {
    world.commands().entity(context.entity).insert(QueuedSample);
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
    ///
    /// This immediately queues up the sample for playback.
    pub fn new(handle: Handle<Sample>) -> Self {
        Self {
            sample: handle,
            player: None,
        }
    }

    /// Get a shared reference to the inner sample handle.
    pub fn sample(&self) -> &Handle<Sample> {
        &self.sample
    }

    /// Returns whether this sample is currently playing.
    pub fn is_playing(&self) -> bool {
        self.player
            .as_ref()
            .map(|p| !p.state.stopped())
            .unwrap_or_default()
    }

    /// Returns the current playhead in frames.
    ///
    /// If this sample player has not yet been assigned to a pool,
    /// this returns `None`.
    pub fn playhead_frames(&self) -> Option<u64> {
        self.player.as_ref().map(|p| p.state.playhead_frames())
    }

    pub(crate) fn set_sampler(&mut self, entity: Entity, state: SamplerState) {
        self.player = Some(Player { state, entity });
    }

    pub(crate) fn clear_sampler(&mut self) {
        self.player = None;
    }
}

#[derive(Clone)]
struct Player {
    state: SamplerState,
    entity: Entity,
}

impl core::fmt::Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Player")
            .field("entity", &self.entity)
            .finish_non_exhaustive()
    }
}

/// Controls the playback settings of a [`SamplePlayer`].
///
/// `repeate_mode` and `volume` are read _once_ at the beginning
/// of playback. Changing them during playback will not
/// affect playback.
#[derive(Debug, Component, Clone)]
pub struct PlaybackSettings {
    /// Sets the sample's [`RepeatMode`].
    pub repeat_mode: RepeatMode,

    /// Determines this sample's behavior on playback completion.
    pub on_complete: OnComplete,

    /// Sets the volume of the sample.
    pub volume: Volume,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self::ONCE
    }
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

/// Determines what happens when a sample completes playback.
///
/// This will never trigger for looping samples.
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

/// Sample parameters that can change during playback.
///
/// These parameters will apply to samples immediately, so
/// you can choose to begin playback wherever you'd like,
/// or even start with the sample paused.
///
/// ```
/// # use bevy_seedling::prelude::*;
/// # use bevy::prelude::*;
/// fn play_with_params(mut commands: Commands, server: Res<AssetServer>) {
///     commands.spawn((
///         SamplePlayer::new(server.load("my_sample.wav")),
///         // You can start one second in
///         PlaybackParams {
///             playhead: Notify::new(Playhead::Seconds(1.0)),
///             ..Default::default()
///         },
///     ));
///
///     commands.spawn((
///         SamplePlayer::new(server.load("my_sample.wav")),
///         // Or even spawn with paused playback
///         PlaybackParams {
///             playback: Notify::new(PlaybackState::Pause),
///             ..Default::default()
///         },
///     ));
/// }
/// ```
#[derive(Component, Debug)]
pub struct PlaybackParams {
    /// Sets the playback state, allowing you to play, pause or stop samples.
    ///
    /// This field provides only one-way communication with the
    /// audio processor. To get whether the sample is playing,
    /// see [`SamplePlayer::is_playing`].
    pub playback: Notify<PlaybackState>,

    /// Sets the playhead.
    ///
    /// This field provides only one-way communication with the
    /// audio processor. To get the current value of the playhead,
    /// see [`SamplePlayer::playhead_frames`].
    pub playhead: Notify<Playhead>,

    /// Sets the playback speed.
    pub speed: f64,
}

impl PlaybackParams {
    /// Start or resume playback.
    ///
    /// ```
    /// # use bevy_seedling::prelude::*;
    /// # use bevy::prelude::*;
    /// fn resume_paused_samples(mut samples: Query<&mut PlaybackParams>) {
    ///     for mut params in samples.iter_mut() {
    ///         if !matches!(*params.playback, PlaybackState::Play { .. }) {
    ///             params.play();
    ///         }
    ///     }
    /// }
    /// ```
    pub fn play(&mut self) {
        *self.playback = PlaybackState::Play { delay: None };
    }

    /// Pause playback.
    ///
    /// ```
    /// # use bevy_seedling::prelude::*;
    /// # use bevy::prelude::*;
    /// fn pause_all_samples(mut samples: Query<&mut PlaybackParams>) {
    ///     for mut params in samples.iter_mut() {
    ///         params.pause();
    ///     }
    /// }
    /// ```
    pub fn pause(&mut self) {
        *self.playback = PlaybackState::Pause;
    }

    /// Stop playback, resetting the playhead to the start.
    ///
    /// ```
    /// # use bevy_seedling::prelude::*;
    /// # use bevy::prelude::*;
    /// fn stop_all_samples(mut samples: Query<&mut PlaybackParams>) {
    ///     for mut params in samples.iter_mut() {
    ///         params.stop();
    ///     }
    /// }
    /// ```
    pub fn stop(&mut self) {
        *self.playback = PlaybackState::Stop;
        *self.playhead = Playhead::default();
    }
}

impl Default for PlaybackParams {
    fn default() -> Self {
        Self {
            playback: Notify::new(PlaybackState::Play { delay: None }),
            playhead: Notify::default(),
            speed: 1.0,
        }
    }
}

/// A marker struct for entities that are waiting
/// for asset loading and playback assignment.
#[derive(Debug, Component, Default)]
#[component(storage = "SparseSet")]
pub struct QueuedSample;
