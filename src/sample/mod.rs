//! Audio sample components.

use std::time::Duration;

use crate::prelude::Volume;
use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};
use firewheel::{
    diff::Notify,
    nodes::sampler::{self, PlaybackState, Playhead, RepeatMode},
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
/// This queues playback in a [`SamplerPool`][crate::prelude::SamplerPool].
/// When no effects are applied, samples are played in the
/// [`DefaultPool`][crate::prelude::DefaultPool].
///
/// Playback is managed with two components: an immutable [`PlaybackStatic`]
/// component, read once at the beginning of playback, and a [`PlaybackDynamic`]
/// component, which can be updated dynamically.
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
#[require(PlaybackSettings, SamplePriority, SampleQueueLifetime)]
#[component(on_insert = on_insert_sample, immutable)]
pub struct SamplePlayer {
    /// The sample to play.
    pub sample: Handle<Sample>,

    /// Sets the sample's [`RepeatMode`].
    pub repeat_mode: RepeatMode,

    /// Sets the volume of the sample.
    pub volume: Volume,
}

fn example(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn(SamplePlayer::new(server.load("caw.ogg")));

    commands.spawn(
        SamplePlayer::new(server.load("caw.ogg"))
            .looping()
            .with_volume(Volume::Decibels(-6.0)),
    );

    commands.spawn(SamplePlayer {
        sample: server.load("caw.ogg"),
        ..Default::default()
    });

    commands.spawn(SamplePlayer {
        sample: server.load("caw.ogg"),
        repeat_mode: RepeatMode::RepeatEndlessly,
        volume: Volume::Decibels(-6.0),
    });
}

impl Default for SamplePlayer {
    fn default() -> Self {
        Self {
            sample: Default::default(),
            repeat_mode: RepeatMode::PlayOnce,
            volume: Volume::UNITY_GAIN,
        }
    }
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
            ..Default::default()
        }
    }

    pub fn looping(self) -> Self {
        Self {
            repeat_mode: RepeatMode::RepeatEndlessly,
            ..self
        }
    }

    pub fn with_volume(self, volume: Volume) -> Self {
        Self { volume, ..self }
    }
}

#[derive(Component, Clone)]
#[component(immutable)]
pub struct SampleState(pub(crate) sampler::SamplerState);

impl core::fmt::Debug for SampleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SamplerState").finish_non_exhaustive()
    }
}

impl SampleState {
    /// Returns whether this sample is currently playing.
    pub fn is_playing(&self) -> bool {
        !self.0.stopped()
    }

    /// Returns the current playhead in frames.
    pub fn playhead_frames(&self) -> u64 {
        self.0.playhead_frames()
    }
}

#[derive(Debug, Default, Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[component(immutable)]
pub struct SamplePriority(pub u32);

/// The maximum duration of time that a sample will wait for an available sampler.
///
/// The timer begins once the sample asset has loaded and after the sample player has been skipped
/// at least once. If the sample player is not queued for playback within this duration,
/// it will be considered to have completed playback.
///
/// The default lifetime is 100ms.
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SampleQueueLifetime(pub Duration);

impl Default for SampleQueueLifetime {
    fn default() -> Self {
        Self(Duration::from_millis(100))
    }
}

///// Controls the playback settings of a [`SamplePlayer`].
/////
///// `repeate_mode` and `volume` are read _once_ at the beginning
///// of playback. Changing them during playback will not
///// affect playback.
//#[derive(Debug, Component, Clone)]
//pub struct PlaybackStatic {
//    /// Sets the sample's [`RepeatMode`].
//    pub repeat_mode: RepeatMode,
//
//    /// Determines this sample's behavior on playback completion.
//    pub on_complete: OnComplete,
//
//    /// Sets the volume of the sample.
//    pub volume: Volume,
//}
//
//impl Default for PlaybackStatic {
//    fn default() -> Self {
//        Self::ONCE
//    }
//}
//
//impl PlaybackStatic {
//    /// Play the audio source once, despawning
//    /// this entity when complete or interrupted.
//    pub const ONCE: Self = Self {
//        repeat_mode: RepeatMode::PlayOnce,
//        volume: Volume::Linear(1.0),
//        on_complete: OnComplete::Despawn,
//    };
//
//    /// Repeatedly loop the audio source until
//    /// this entity is despawned.
//    pub const LOOP: Self = Self {
//        repeat_mode: RepeatMode::RepeatEndlessly,
//        volume: Volume::Linear(1.0),
//        on_complete: OnComplete::Despawn,
//    };
//
//    /// Play the sample once, removing the audio-related components on completion.
//    pub const REMOVE: Self = Self {
//        repeat_mode: RepeatMode::PlayOnce,
//        volume: Volume::Linear(1.0),
//        on_complete: OnComplete::Remove,
//    };
//
//    /// Play the sample once, preserving the components and entity on completion.
//    pub const PRESERVE: Self = Self {
//        repeat_mode: RepeatMode::PlayOnce,
//        volume: Volume::Linear(1.0),
//        on_complete: OnComplete::Preserve,
//    };
//}
//

/// Determines what happens when a sample completes playback.
///
/// This will not trigger for looping samples unless they are stopped.
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
pub struct PlaybackSettings {
    /// Sets the playback state, allowing you to play, pause or stop samples.
    ///
    /// This field provides only one-way communication with the
    /// audio processor. To get whether the sample is playing,
    /// see [`SamplerState::is_playing`].
    pub playback: Notify<PlaybackState>,

    /// Sets the playhead.
    ///
    /// This field provides only one-way communication with the
    /// audio processor. To get the current value of the playhead,
    /// see [`SamplerState::playhead_frames`].
    pub playhead: Notify<Playhead>,

    /// Sets the playback speed.
    pub speed: f64,

    /// Determines this sample's behavior on playback completion.
    pub on_complete: OnComplete,
}

impl PlaybackSettings {
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

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            playback: Notify::new(PlaybackState::Play { delay: None }),
            playhead: Notify::default(),
            speed: 1.0,
            on_complete: OnComplete::Remove,
        }
    }
}

/// A marker struct for entities that are waiting
/// for asset loading and playback assignment.
#[derive(Debug, Component, Default)]
#[component(storage = "SparseSet")]
pub struct QueuedSample;

#[cfg(feature = "rand")]
pub use random::PitchRange;

#[cfg(feature = "rand")]
pub(crate) use random::RandomPlugin;

#[cfg(feature = "rand")]
mod random {
    use super::PlaybackSettings;
    use bevy::{
        ecs::{component::HookContext, world::DeferredWorld},
        prelude::*,
    };
    use rand::{Rng, SeedableRng, rngs::SmallRng};

    pub struct RandomPlugin;

    impl Plugin for RandomPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(PitchRng(SmallRng::from_entropy()));
        }
    }

    #[derive(Resource)]
    struct PitchRng(SmallRng);

    /// A component that applies a random pitch
    /// to a sample player when spawned.
    #[derive(Debug, Component, Default, Clone)]
    #[require(PlaybackSettings)]
    #[component(immutable, on_add = Self::on_add_hook)]
    pub struct PitchRange(pub core::ops::Range<f64>);

    impl PitchRange {
        fn on_add_hook(mut world: DeferredWorld, context: HookContext) {
            let range = world
                .get::<PitchRange>(context.entity)
                .expect("Entity should have a `PitchRange` component")
                .0
                .clone();

            let mut rng = world.resource_mut::<PitchRng>();
            let value = rng.0.gen_range(range);

            world
                .commands()
                .entity(context.entity)
                .entry::<PlaybackSettings>()
                .or_default()
                .and_modify(move |mut params| params.speed = value);
        }
    }
}
