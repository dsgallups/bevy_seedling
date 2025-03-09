use super::{label::PoolLabelContainer, PlaybackSettings, QueuedSample, Sample, SamplePlayer};
use crate::{node::Events, ConnectNode, DefaultPool, PoolLabel, SeedlingSystems};
use bevy_app::{Last, Plugin};
use bevy_asset::Assets;
use bevy_ecs::{component::ComponentId, prelude::*, world::DeferredWorld};
use bevy_hierarchy::{BuildChildren, DespawnRecursiveExt};
use firewheel::diff::{Diff, Patch, PathBuilder};
use firewheel::event::{NodeEventType, SequenceCommand};
use firewheel::node::AudioNode;
use firewheel::nodes::sampler::SamplerNode;

pub(crate) struct SamplePoolPlugin;

impl Plugin for SamplePoolPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(
            Last,
            (
                (remove_finished, assign_default)
                    .before(SeedlingSystems::Queue)
                    .after(SeedlingSystems::Acquire),
                monitor_active
                    .before(SeedlingSystems::Flush)
                    .after(SeedlingSystems::Queue),
            ),
        );
    }
}

/// A sampler pool builder.
#[derive(Debug)]
pub struct Pool<L, C> {
    label: L,
    size: usize,
    effects_chain: C,
}

impl<L: PoolLabel + Component + Clone> Pool<L, ()> {
    pub fn new(label: L, size: usize) -> Self {
        Self {
            label,
            size,
            effects_chain: (),
        }
    }
}

pub trait ExtendTuple {
    type Output<T>;

    fn extend<T>(self, value: T) -> Self::Output<T>;
}

macro_rules! extend {
    ($($A:ident),*) => {
        impl<$($A),*> ExtendTuple for ($($A,)*) {
            type Output<Z> = ($($A,)* Z,);

            #[allow(non_snake_case)]
            fn extend<Z>(self, value: Z) -> Self::Output<Z> {
                let ($($A,)*) = self;

                ($($A,)* value,)
            }
        }
    };
}

bevy_utils::all_tuples!(extend, 0, 15, A);

impl<L, C: ExtendTuple> Pool<L, C> {
    pub fn effect<T: AudioNode + Clone + Component>(self, node: T) -> Pool<L, C::Output<T>> {
        Pool {
            label: self.label,
            size: self.size,
            effects_chain: self.effects_chain.extend(node),
        }
    }
}

macro_rules! spawn_impl {
    ($($ty:ident),*) => {
        impl<L: PoolLabel + Component + Clone, $($ty),*> Pool<L, ($($ty,)*)>
        where $($ty: Component + Clone),*
        {
            #[allow(non_snake_case)]
            pub fn spawn<'a>(
                self,
                commands: &'a mut Commands,
            ) -> EntityCommands<'a> {
                let Self {
                    label,
                    size,
                    effects_chain,
                } = self;

                commands.queue(|world: &mut World| {
                    world.schedule_scope(Last, |_, schedule| {
                        schedule.add_systems(
                            (rank_nodes::<L>, assign_work::<L>)
                                .chain()
                                .in_set(SeedlingSystems::Queue),
                        );
                    });
                });

                let bus = commands
                    .spawn((
                        crate::VolumeNode {
                            normalized_volume: 1.0,
                        },
                        SamplePoolNode,
                        label.clone(),
                        SamplePoolDefaults(Box::new({
                            let effects_chain = effects_chain.clone();
                            #[allow(unused)]
                            move |commands: &mut EntityCommands| {
                                let ($($ty,)*) = &effects_chain;

                                $(commands.entry::<$ty>().or_insert($ty.clone());)*
                            }
                        })),
                    ))
                    .id();

                let rank = commands.spawn((NodeRank::default(), label.clone())).id();

                let ($($ty,)*) = effects_chain;

                let nodes: Vec<_> = (0..size)
                    .map(|_| {
                        let chain: Vec<Entity> = vec![$(
                            commands.spawn((
                                <$ty as ::core::clone::Clone>::clone(&$ty),
                                SamplePoolNode,
                                label.clone()
                            )).id(),
                        )*];

                        let source = commands
                            .spawn((SamplerNode::default(), SamplePoolNode, label.clone(), EffectsChain(chain.clone())))
                            .add_children(&chain)
                            .id();

                        let mut chain = chain;
                        chain.push(bus);

                        commands.entity(source).connect(chain[0]);

                        for pair in chain.windows(2) {
                            commands.entity(pair[0]).connect(pair[1]);
                        }

                        source
                    })
                    .collect();

                let mut bus = commands.entity(bus);
                bus.add_children(&nodes).add_child(rank);

                bus
            }
        }
    };
}

bevy_utils::all_tuples!(spawn_impl, 0, 15, A);

///// Provides methods on [`Commands`] to spawn new sample pools.
//pub trait SpawnPool {
//    /// Spawn a sample pool, returning the [`EntityCommands`] for
//    /// the terminal volume node.
//    ///
//    /// ```
//    /// # use bevy::prelude::*;
//    /// use bevy_seedling::{SpawnPool, PoolLabel, SamplePlayer};
//    ///
//    /// #[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
//    /// struct CustomPool;
//    ///
//    /// fn spawn_custom_pool(server: Res<AssetServer>, mut commands: Commands) {
//    ///     // Spawn a custom sample pool
//    ///     commands.spawn_pool(CustomPool, 16);
//    ///
//    ///     // Trigger sample playback in the custom pool
//    ///     commands.spawn((
//    ///         SamplePlayer::new(server.load("my_sample.wav")),
//    ///         CustomPool,
//    ///     ));
//    /// }
//    /// ```
//    fn pool(&mut self) -> PoolBuilder<()>;
//
//    ///// Spawn a sample pool with an initial volume, returning the [`EntityCommands`] for
//    ///// the terminal volume node.
//    //fn spawn_pool_with<T: PoolLabel + Component + Clone>(
//    //    &mut self,
//    //    marker: T,
//    //    size: usize,
//    //    volume: f32,
//    //) -> PoolBuilder<()>;
//    //
//
//    /// Despawn a sample pool, cleaning up its resources
//    /// in the ECS and audio graph.
//    ///
//    /// Despawning the terminal volume node recursively
//    /// will produce the same effect.
//    fn despawn_pool<T: PoolLabel + Component>(&mut self);
//}
//
//impl SpawnPool for Commands<'_, '_> {
//    fn pool(&mut self) -> PoolBuilder<()> {
//        // self.queue(|world: &mut World| {
//        //     world.schedule_scope(Last, |_, schedule| {
//        //         schedule.add_systems(
//        //             (rank_nodes::<T>, assign_work::<T>)
//        //                 .chain()
//        //                 .in_set(SeedlingSystems::Queue),
//        //         );
//        //     });
//        // });
//        //
//        // let bus = self
//        //     .spawn((
//        //         crate::VolumeNode {
//        //             normalized_volume: 1.0,
//        //         },
//        //         SamplePoolNode,
//        //         marker.clone(),
//        //     ))
//        //     .id();
//        //
//        // let rank = self.spawn((NodeRank::default(), marker.clone())).id();
//        //
//        // let nodes: Vec<_> = (0..size)
//        //     .map(|_| {
//        //         self.spawn((SamplerNode::default(), SamplePoolNode, marker.clone()))
//        //             .id()
//        //     })
//        //     .collect();
//        //
//        // self.entity(bus).add_children(&nodes).add_child(rank);
//
//        PoolBuilder {
//            commands: self.reborrow(),
//            effects_chain: (),
//        }
//    }
//
//    fn despawn_pool<T: PoolLabel + Component>(&mut self) {
//        self.queue(|world: &mut World| {
//            let mut roots = world
//                .query_filtered::<Entity, (With<T>, With<SamplePoolNode>, With<crate::VolumeNode>)>(
//                );
//
//            let roots: Vec<_> = roots.iter(world).collect();
//
//            let mut commands = world.commands();
//
//            for root in roots {
//                commands.entity(root).despawn_recursive();
//            }
//        });
//    }
//}

#[derive(Component)]
struct SamplePoolNode;

#[derive(Component)]
struct EffectsChain(Vec<Entity>);

#[derive(Component)]
struct SamplePoolDefaults(Box<dyn Fn(&mut EntityCommands) + Send + Sync + 'static>);

#[derive(Default, Component)]
struct NodeRank(Vec<(Entity, u64)>);

fn rank_nodes<T: Component>(
    q: Query<(Entity, &SamplerNode), (With<SamplePoolNode>, With<T>)>,
    mut rank: Query<&mut NodeRank, With<T>>,
) {
    let Ok(mut rank) = rank.get_single_mut() else {
        return;
    };

    rank.0.clear();

    for (e, params) in q.iter() {
        let score = params.worker_score();

        rank.0.push((e, score));
    }

    rank.0
        .sort_unstable_by_key(|pair| std::cmp::Reverse(pair.1));
}

// fn supply_defaults<T: Component>(
//     defaults: Query<&SamplePoolDefaults, With<T>>,
//     q: Query<Entity, (With<SamplePlayer>, Added<T>)>,
// ) {
//
// }

#[derive(Component, Clone, Copy)]
#[component(on_remove = on_remove_active)]
struct ActiveSample {
    sample_entity: Entity,
    despawn: bool,
}

fn on_remove_active(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
    let active = *world.entity(entity).components::<&ActiveSample>();

    if active.despawn {
        if let Some(commands) = world.commands().get_entity(active.sample_entity) {
            commands.despawn_recursive();
        }
    }
}

fn remove_finished(
    nodes: Query<(Entity, &SamplerNode, &EffectsChain), With<ActiveSample>>,
    mut commands: Commands,
) {
    for (entity, sampler, effects_chain) in nodes.iter() {
        let state = sampler.playback_state();

        // TODO: this will remove samples when paused
        if !state.is_playing() {
            commands.entity(entity).remove::<ActiveSample>();

            for effect in effects_chain.0.iter() {
                commands.entity(*effect).remove::<ParamFollower>();
            }
        }
    }
}

fn assign_work<T: Component>(
    mut nodes: Query<
        (Entity, &mut SamplerNode, &mut Events, &EffectsChain),
        (With<SamplePoolNode>, With<T>),
    >,
    queued_samples: Query<
        (Entity, &SamplePlayer, &PlaybackSettings),
        (With<QueuedSample>, With<T>),
    >,
    mut rank: Query<&mut NodeRank, With<T>>,
    defaults: Query<&SamplePoolDefaults, With<T>>,
    assets: Res<Assets<Sample>>,
    mut commands: Commands,
) {
    let Ok(mut rank) = rank.get_single_mut() else {
        return;
    };

    for (sample, player, settings) in queued_samples.iter() {
        let Some(asset) = assets.get(&player.0) else {
            continue;
        };

        // get the best candidate
        let Some((node_entity, _)) = rank.0.first() else {
            continue;
        };

        let Ok((node_entity, mut params, mut events, effects_chain)) = nodes.get_mut(*node_entity)
        else {
            continue;
        };

        params.set_sample(asset.get(), settings.volume, settings.mode);
        let event = params.sync_params_event(true);
        events.push(event);

        // redirect all parameters to follow the sample source
        for effect in effects_chain.0.iter() {
            commands.entity(*effect).insert(ParamFollower(sample));
        }

        // Insert default pool parameters if not present.
        if let Ok(defaults) = defaults.get_single() {
            (defaults.0)(&mut commands.entity(sample));
        }

        rank.0.remove(0);
        commands.entity(sample).remove::<QueuedSample>();
        commands.entity(node_entity).insert(ActiveSample {
            sample_entity: sample,
            despawn: true,
        });
    }
}

/// A component that allows one entity's parameters to track another's.
///
/// This can only support a single relationship; cascading
/// is not allowed.
#[derive(Component)]
pub(crate) struct ParamFollower(Entity);

pub(crate) fn param_follower<T: Diff + Patch + Component>(
    sources: Query<&T, Without<ParamFollower>>,
    mut followers: Query<(&ParamFollower, &mut T)>,
) {
    let mut event_queue = Vec::new();
    for (follower, mut params) in followers.iter_mut() {
        let Ok(source) = sources.get(follower.0) else {
            continue;
        };

        source.diff(&params, PathBuilder::default(), &mut event_queue);

        for event in event_queue.drain(..) {
            params.patch_event(&event);
        }
    }
}

// Stop playback if the source entity no longer exists.
fn monitor_active(
    mut nodes: Query<(Entity, &ActiveSample, &mut Events, &EffectsChain)>,
    samples: Query<&SamplePlayer>,
    mut commands: Commands,
) {
    for (node_entity, active, mut events, effects_chain) in nodes.iter_mut() {
        if samples.get(active.sample_entity).is_err() {
            events.push(NodeEventType::SequenceCommand(SequenceCommand::Stop));

            commands.entity(node_entity).remove::<ActiveSample>();

            for effect in effects_chain.0.iter() {
                commands.entity(*effect).remove::<ParamFollower>();
            }
        }
    }
}

fn assign_default(
    samples: Query<Entity, (With<SamplePlayer>, Without<PoolLabelContainer>)>,
    mut commands: Commands,
) {
    for sample in samples.iter() {
        commands.entity(sample).insert(DefaultPool);
    }
}
