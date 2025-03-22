//! Dynamic sampler pools.
//!
//! *Sampler pools* are `bevy_seedling`'s primary mechanism for playing
//! multiple sounds at once. [`PoolBuilder`] allows you to create these pools on-the-fly.
//!
//! [`PoolBuilder`] is implemented on [`EntityCommands`], so you'll typically apply effects
//! to an entity after spawning it.
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_seedling::prelude::*;
//! fn effects(mut commands: Commands, server: Res<AssetServer>) {
//!     commands
//!         .spawn(SamplePlayer::new(server.load("my_sample.wav")))
//!         .effect(SpatialBasicNode::default())
//!         .effect(LowPassNode::new(500.0));
//! }
//! ```
//!
//! In the above example, we connect a spatial and low-pass node in series with the sample player.
//! Effects are arranged in the order of `effect` calls, so the output of the spatial node is
//! connected to the input of the low-pass node.
//!
//! Once per frame, `bevy_seedling` will scan for [`SamplePlayer`]s that request dynamic pools, assigning
//! the sample to an existing dynamic pool or creating a new one if none match. The number of
//! samplers in a dynamic pool is determined by
//! [`SeedlingPlugin::dynamic_pool_range`][crate::SeedlingPlugin::dynamic_pool_range].
//! The pool is spawned with the range's `start` value, and as demand increases, the pool
//! grows until the range's `end`.
//!
//! ## When to use dynamic pools
//!
//! Dynamic pools are a convenient abstraction, but they may not be appropriate for all use-cases.
//! They have three main drawbacks:
//!
//! 1. Dynamic pools cannot be routed anywhere.
//! 2. The number of pools corresponds to the total permutations of effects your project uses,
//!    which could grow fairly large. Silent sampler nodes shouldn't take much CPU time,
//!    but many unused nodes could grow your memory usage by a few megabytes.
//! 3. Dynamic pools are spawned on-the-fly, so you may see up to a frame of additional
//!    playback latency as the pool propagates to the audio graph.
//!
//! Dynamic pool are best-suited for sounds that do not need complicated routing or
//! bus configurations and when the kinds of effects you apply are simple and regular.
//! Keep in mind that you can freely mix dynamic and static pools, so you're not restricted
//! to only one or the other!
//!
//! Note that when no effects are applied, your samples will be queued in the
//! [`DefaultPool`][crate::prelude::DefaultPool], not a dynamic pool.

use super::{builder::PoolBuilder, SamplePoolDefaults};
use crate::sample::{QueuedSample, SamplePlayer};
use bevy_ecs::{component::ComponentId, prelude::*, world::DeferredWorld};
use bevy_utils::HashMap;
use core::marker::PhantomData;
use firewheel::node::AudioNode;
use seedling_macros::PoolLabel;

#[derive(Component, Clone, Debug, Eq)]
pub(crate) struct DynamicPoolRegistry {
    effects: Vec<ComponentId>,
}

impl PartialEq for DynamicPoolRegistry {
    fn eq(&self, other: &Self) -> bool {
        self.effects == other.effects
    }
}

impl core::hash::Hash for DynamicPoolRegistry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.effects.hash(state)
    }
}

impl DynamicPoolRegistry {
    pub fn insert(&mut self, value: ComponentId) -> bool {
        if !self.effects.iter().any(|v| *v == value) {
            self.effects.push(value);
            true
        } else {
            false
        }
    }
}

struct RegistryEntry {
    label: DynamicPoolId,
}

#[derive(Resource, Default)]
pub(super) struct Registries(HashMap<DynamicPoolRegistry, RegistryEntry>);

/// Sets the range for the number dynamic pool sampler nodes.
///
/// When the inner value is `None`, no new dynamic pools will be created.
#[derive(Resource, Clone, Debug)]
pub struct DynamicPoolRange(pub Option<core::ops::RangeInclusive<usize>>);

#[derive(PoolLabel, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct DynamicPoolId(usize);

/// Scan through the set of pending sample players
/// and assign work to the most appropriate sampler node.
pub(super) fn update_auto_pools(
    queued_samples: Query<
        (Entity, &DynamicPoolRegistry, &SamplePoolDefaults),
        (
            With<QueuedSample>,
            With<SamplePlayer>,
            Without<DynamicPoolId>,
        ),
    >,
    mut registries: ResMut<Registries>,
    mut commands: Commands,
    dynamic_range: Res<DynamicPoolRange>,
) {
    let Some(dynamic_range) = dynamic_range.0.clone() else {
        return;
    };

    for (sample, registry, defaults) in queued_samples.iter() {
        match registries.0.get_mut(registry) {
            Some(entry) => {
                commands.entity(sample).insert(entry.label);
            }
            None => {
                let label = DynamicPoolId(registries.0.len());

                // create the pool
                super::spawn_pool(
                    label,
                    dynamic_range.clone(),
                    defaults.clone(),
                    &mut commands,
                );

                registries
                    .0
                    .insert(registry.clone(), RegistryEntry { label });

                commands.entity(sample).insert(label);
            }
        }
    }
}

// NOTE: I'd prefer not to use this since there could be a decorrelation
// between this and the defaults, but it's a touch tricky to get at the
// `ComponentId` without some world access.
#[derive(Component)]
#[component(on_insert = Self::on_insert)]
pub(crate) struct AutoRegister<T: Component>(PhantomData<T>);

impl<T: Component> core::default::Default for AutoRegister<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: Component> AutoRegister<T> {
    fn on_insert(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        let Some(id) = world.component_id::<T>() else {
            return;
        };

        let mut entity = world.entity_mut(entity);

        if let Some(mut pool) = entity.get_mut::<DynamicPoolRegistry>() {
            pool.insert(id);
        }
    }
}

/// A wrapper around [`EntityCommands`] for applying audio effects.
///
/// For more information, see [the module docs][self].
pub struct DynamicPoolCommands<'a> {
    commands: EntityCommands<'a>,
}

impl core::fmt::Debug for DynamicPoolCommands<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicPoolCommands")
            .finish_non_exhaustive()
    }
}

impl<'a> PoolBuilder for EntityCommands<'a> {
    type Output = DynamicPoolCommands<'a>;

    fn effect<T: AudioNode + Component + Clone>(mut self, node: T) -> Self::Output {
        let mut defaults = SamplePoolDefaults::default();

        defaults.push(node.clone());

        self.insert((
            DynamicPoolRegistry {
                effects: Default::default(),
            },
            defaults,
            node,
        ));

        DynamicPoolCommands { commands: self }
    }
}

impl<'a> PoolBuilder for DynamicPoolCommands<'a> {
    type Output = DynamicPoolCommands<'a>;

    fn effect<T: AudioNode + Component + Clone>(mut self, node: T) -> Self::Output {
        self.commands
            .entry::<SamplePoolDefaults>()
            .or_default()
            .and_modify({
                let node = node.clone();
                move |mut defaults| {
                    defaults.push(node);
                }
            });
        self.commands.insert(node);

        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        pool::{NodeRank, SamplerNodes},
        prelude::*,
        profiling::ProfilingBackend,
    };
    use bevy::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    fn prepare_app<F: IntoSystem<(), (), M>, M>(startup: F) -> App {
        let mut app = App::new();

        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            SeedlingPlugin::<ProfilingBackend> {
                default_pool_size: None,
                dynamic_pool_range: Some(4..=16),
                ..SeedlingPlugin::<ProfilingBackend>::new()
            },
            HierarchyPlugin,
        ))
        .add_systems(Startup, startup);

        app.finish();
        app.cleanup();
        app.update();

        app
    }

    fn run<F: IntoSystem<(), O, M>, O, M>(app: &mut App, system: F) -> O {
        let world = app.world_mut();
        world.run_system_once(system).unwrap()
    }

    #[test]
    fn test_dynamic_pool_shape() {
        let mut app = prepare_app(|mut commands: Commands, server: Res<AssetServer>| {
            commands
                .spawn(SamplePlayer::new(server.load("caw.ogg")))
                .effect(LowPassNode::default());
        });

        fn verify_one_pool(pool_root: Query<(&DynamicPoolId, &SamplerNodes), With<NodeRank>>) {
            let (id, children) = pool_root.single();

            assert_eq!(id.0, 0);
            assert_eq!(children.len(), 4);
        }

        run(&mut app, verify_one_pool);

        // We'll spawn another player to ensure we don't create a new pool.
        run(
            &mut app,
            |mut commands: Commands, server: Res<AssetServer>| {
                commands
                    .spawn(SamplePlayer::new(server.load("caw.ogg")))
                    .effect(LowPassNode::default());
            },
        );

        app.update();

        run(&mut app, verify_one_pool);

        // Now we'll spawn a different pool.
        run(
            &mut app,
            |mut commands: Commands, server: Res<AssetServer>| {
                commands
                    .spawn(SamplePlayer::new(server.load("caw.ogg")))
                    .effect(BandPassNode::default());
            },
        );

        app.update();

        run(
            &mut app,
            |pool_root: Query<(&DynamicPoolId, &SamplerNodes), With<NodeRank>>| {
                assert_eq!(pool_root.iter().count(), 2);
            },
        );
    }

    #[test]
    fn test_dynamic_pool_bounds() {
        let mut app = prepare_app(|mut commands: Commands, server: Res<AssetServer>| {
            commands
                .spawn(SamplePlayer::new(server.load("caw.ogg")))
                .effect(LowPassNode::default());
        });

        run(
            &mut app,
            |pool_root: Query<(&DynamicPoolId, &SamplerNodes), With<NodeRank>>| {
                let (_, children) = pool_root.single();

                assert_eq!(children.len(), 4);
            },
        );

        run(
            &mut app,
            |mut commands: Commands, server: Res<AssetServer>| {
                for _ in 0..5 {
                    commands
                        .spawn(SamplePlayer::new(server.load("caw.ogg")))
                        .effect(LowPassNode::default());
                }
            },
        );

        app.update();

        // wait until the samples are loaded
        loop {
            if run(
                &mut app,
                |players: Query<&SamplePlayer>, server: Res<AssetServer>| {
                    let first = players.iter().next().unwrap();

                    server.is_loaded(&first.0)
                },
            ) {
                break;
            }

            app.update();
        }

        app.update();

        run(
            &mut app,
            |pool_root: Query<(&DynamicPoolId, &SamplerNodes), With<NodeRank>>,
             players: Query<&SamplePlayer>| {
                assert_eq!(players.iter().count(), 6);

                let (_, children) = pool_root.single();
                assert!(children.len() > 4);
            },
        );
    }
}
