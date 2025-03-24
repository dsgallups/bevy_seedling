//! Type-based sample pool labeling.
//!
//! `bevy_seedling` provides a single pool label, [`DefaultPool`].
//! Any node that doesn't provide an explicit pool when spawned
//! and has no effects will be automatically played in the [`DefaultPool`].
//!
//! You can customize the default sampler pool by setting
//! [`SeedlingPlugin::default_pool_size`][crate::prelude::SeedlingPlugin::default_pool_size]
//! to `None`, preventing the plugin from spawning it for you.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_seedling::prelude::*;
//!
//! fn main() {
//!     App::default()
//!         .add_plugins((
//!             DefaultPlugins,
//!             SeedlingPlugin {
//!                 default_pool_size: None,
//!                 ..Default::default()
//!             },
//!         ))
//!         .add_systems(Startup, |mut commands: Commands| {
//!             // Make the default pool provide spatial audio
//!             Pool::new(DefaultPool, 24)
//!                 .effect(SpatialBasicNode::default())
//!                 .spawn(&mut commands);
//!         })
//!         .run();
//! }
//! ```

use bevy_ecs::{component::ComponentId, intern::Interned, prelude::*, world::DeferredWorld};

pub use seedling_macros::PoolLabel;

bevy_ecs::define_label!(
    /// A label for differentiating sample pools.
    ///
    /// When deriving [`PoolLabel`], you'll need to make sure your type implements
    /// a few additional traits.
    ///
    /// ```
    /// # use bevy_seedling::prelude::*;
    /// #[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
    /// struct MyPool;
    /// ```
    PoolLabel,
    POOL_LABEL_INTERNER
);

/// The default sample pool.
///
/// If no pool is specified when spawning a
/// [`SamplePlayer`] and no effects are applied,
/// this label will be inserted automatically.
///
/// [`SamplePlayer`]: crate::sample::SamplePlayer
///
/// You can customize the default sampler pool by setting
/// [`SeedlingPlugin::default_pool_size`][crate::prelude::SeedlingPlugin::default_pool_size]
/// to `None`, preventing the plugin from spawning it for you.
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_seedling::prelude::*;
///
/// fn main() {
///     App::default()
///         .add_plugins((
///             DefaultPlugins,
///             SeedlingPlugin {
///                 default_pool_size: None,
///                 ..Default::default()
///             },
///         ))
///         .add_systems(Startup, |mut commands: Commands| {
///             // Make the default pool provide spatial audio
///             Pool::new(DefaultPool, 24)
///                 .effect(SpatialBasicNode::default())
///                 .spawn(&mut commands);
///         })
///         .run();
/// }
/// ```
#[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefaultPool;

/// A type-erased node label.
pub type InternedPoolLabel = Interned<dyn PoolLabel>;

/// A type-erased pool label container.
#[derive(Component, Debug, Clone)]
#[component(on_remove = Self::on_remove)]
pub struct PoolLabelContainer {
    pub(crate) label: InternedPoolLabel,
    label_id: ComponentId,
}

impl PoolLabelContainer {
    /// Create a new interned pool label.
    pub fn new<T: PoolLabel>(label: &T, id: ComponentId) -> Self {
        Self {
            label: label.intern(),
            label_id: id,
        }
    }

    fn on_remove(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        let id = world
            .entity(entity)
            .components::<&PoolLabelContainer>()
            .label_id;
        world.commands().entity(entity).remove_by_id(id);
    }
}
