//! Pool builder trait and struct.

use super::SamplePoolTypes;
use crate::prelude::PoolLabel;
use bevy_ecs::prelude::*;
use firewheel::node::AudioNode;

/// Chain effects in a pool.
///
/// For applying effects directly to [`SamplePlayer`], see the [dynamic][super::dynamic] module.
///
/// For building pools, see [`Pool`].
///
/// [`SamplePlayer`]: crate::prelude::SamplePlayer
pub trait PoolBuilder {
    /// The output, typically `Self`.
    type Output;

    /// Insert an effect into a pool.
    ///
    /// This can be used in dynamic contexts directly on a [`SamplePlayer`] entity.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_seedling::prelude::*;
    /// fn dynamic_context(mut commands: Commands, server: Res<AssetServer>) {
    ///     commands
    ///         .spawn(SamplePlayer::new(server.load("my_sample.wav")))
    ///         .effect(SpatialBasicNode::default())
    ///         .effect(LowPassNode::new(500.0));
    /// }
    /// ```
    ///
    /// Or in static contexts on [`Pool`].
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_seedling::prelude::*;
    /// fn static_context(mut commands: Commands) {
    ///     #[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
    ///     struct SpatialPool;
    ///
    ///     Pool::new(SpatialPool, 4)
    ///         .effect(SpatialBasicNode::default())
    ///         .effect(LowPassNode::new(500.0))
    ///         .spawn(&mut commands);
    /// }
    /// ```
    ///
    /// [`SamplePlayer`]: crate::prelude::SamplePlayer
    fn effect<T: AudioNode + Component + Clone>(self, node: T) -> Self::Output;
}

/// A sample pool builder.
///
///*Sample pools* are `bevy_seedling`'s primary mechanism for playing
/// multiple sounds at once. [`Pool`] allows you to precisely define pools
/// and their routing.
///
/// ## Constructing pools
///
/// To construct a pool, you'll need to provide a unique [`PoolLabel`] and the number
/// of samplers.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_seedling::prelude::*;
/// // Note that you'll need a few additional traits to support `PoolLabel`
/// #[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
/// struct SimplePool;
///
/// fn spawn_pool(mut commands: Commands) {
///     // Here we spawn a simple pool with four sample slots.
///     Pool::new(SimplePool, 4).spawn(&mut commands);
/// }
/// ```
///
/// You can also insert arbitrary effects.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_seedling::prelude::*;
/// # fn spawn_pools(mut commands: Commands) {
/// #[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
/// struct EffectsPool;
///
/// // Here we spawn a pool with effects.
/// Pool::new(EffectsPool, 4)
///     .effect(LowPassNode::default())
///     .effect(SpatialBasicNode::default())
///     .spawn(&mut commands);
/// # }
/// ```
///
/// The `spawn` method returns an [`EntityCommands`], meaning you can easily
/// route the entire pool to arbitrary destinations.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_seedling::prelude::*;
/// # fn spawn_pools(mut commands: Commands) {
/// let filter = commands.spawn(LowPassNode::default()).id();
///
/// #[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
/// struct SimplePool;
///
/// Pool::new(SimplePool, 4)
///     .spawn(&mut commands)
///     .connect(filter);
/// # }
/// ```
///
/// ## Playing samples in a pool
///
/// Once you've spawned a pool, playing samples in it is easy!
/// Just spawn your sample players with the label.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_seedling::prelude::*;
/// #[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
/// struct SimplePool;
///
/// fn spawn_pool(mut commands: Commands) {
///     Pool::new(SimplePool, 4).spawn(&mut commands);
/// }
///
/// fn play_sample(mut commands: Commands, server: Res<AssetServer>) {
///     commands.spawn((SimplePool, SamplePlayer::new(server.load("my_sample.wav"))));
/// }
/// ```
///
/// Pools with effects will automatically insert [*remote nodes*][crate::node::ExcludeNode]
/// for each effect into the [`SamplePlayer`][crate::prelude::SamplePlayer] entity.
/// You can easily override these defaults by including them yourself.
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_seedling::prelude::*;
/// # fn overriding_effects(mut commands: Commands, server: Res<AssetServer>) {
/// #[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
/// struct SpatialPool;
///
/// Pool::new(SpatialPool, 4)
///     .effect(SpatialBasicNode::default())
///     .spawn(&mut commands);
///
/// commands.spawn((
///     SpatialPool,
///     SamplePlayer::new(server.load("my_sample.wav")),
///     SpatialBasicNode {
///         panning_threshold: 0.75,
///         ..Default::default()
///     },
/// ));
/// # }
/// ```
///
/// ## Architecture
///
/// Sample pools are collections of individual
/// sampler nodes, each of which can play a single sample at a time.
/// When samples are queued up for playback, `bevy_seedling` will
/// look for the best sampler in the corresponding pool. If a suitable
/// sampler is found, the sample will begin playback, otherwise
/// waiting until a slot opens up.
///
/// Each sampler node is routed to a final volume node. For a simple pool:
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_seedling::prelude::*;
/// # fn simple_pool(mut commands: Commands) {
/// #[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
/// struct SimplePool;
///
/// Pool::new(SimplePool, 4).spawn(&mut commands);
/// # }
/// ```
///
/// We end up with a graph like:
///
/// ```text
/// ┌───────┐┌───────┐┌───────┐┌───────┐
/// │Sampler││Sampler││Sampler││Sampler│
/// └┬──────┘└┬──────┘└┬──────┘└┬──────┘
/// ┌▽────────▽────────▽────────▽┐
/// │Volume                      │
/// └┬───────────────────────────┘
/// ┌▽──────┐
/// │MainBus│
/// └───────┘
/// ```
///
/// If a pool includes effects, these are inserted in series with each sampler. For a pool
/// with spatial processing:
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_seedling::prelude::*;
/// # fn spatial_pool(mut commands: Commands) {
/// #[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
/// struct SpatialPool;
///
/// Pool::new(SpatialPool, 4)
///     .effect(SpatialBasicNode::default())
///     .spawn(&mut commands);
/// # }
/// ```
///
/// We end up with a graph like:
///
/// ```text
/// ┌───────┐┌───────┐┌───────┐┌───────┐
/// │Sampler││Sampler││Sampler││Sampler│
/// └┬──────┘└┬──────┘└┬──────┘└┬──────┘
/// ┌▽──────┐┌▽──────┐┌▽──────┐┌▽──────┐
/// │Spatial││Spatial││Spatial││Spatial│
/// └┬──────┘└┬──────┘└┬──────┘└┬──────┘
/// ┌▽────────▽────────▽────────▽┐
/// │Volume                      │
/// └┬───────────────────────────┘
/// ┌▽──────┐
/// │MainBus│
/// └───────┘
/// ```
#[derive(Debug)]
pub struct Pool<L> {
    label: L,
    size: usize,
    defaults: SamplePoolTypes,
}

impl<L: PoolLabel + Component + Clone> Pool<L> {
    /// Construct a new [`Pool`].
    ///
    /// Pools are not spawned and propagated to the audio graph
    /// until [`Pool::spawn`] is called.
    ///
    /// A [`Pool`] can be spawned with the same label multiple times,
    /// but the old samplers will be overwritten by the new ones and
    /// all samples queued in the pool will be stopped.
    #[inline(always)]
    #[must_use]
    pub fn new(label: L, size: usize) -> Self {
        Self {
            label,
            size,
            defaults: Default::default(),
        }
    }
}

impl<L: PoolLabel + Component + Clone> Pool<L> {
    /// Spawn the pool, including all its nodes and connections.
    #[inline(always)]
    pub fn spawn<'a>(self, commands: &'a mut Commands) -> EntityCommands<'a> {
        let Self {
            label,
            size,
            defaults,
        } = self;

        super::spawn_pool(label, size..=size, defaults, commands)
    }
}

impl<L> PoolBuilder for Pool<L> {
    type Output = Self;

    #[inline(always)]
    fn effect<T: AudioNode + Component + Clone>(mut self, node: T) -> Self::Output {
        self.defaults.push(node);

        self
    }
}
