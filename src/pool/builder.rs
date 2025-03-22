//! Pool builder trait and struct.

use super::SamplePoolDefaults;
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
#[derive(Debug)]
pub struct Pool<L> {
    label: L,
    size: usize,
    defaults: SamplePoolDefaults,
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
