use bevy::{
    ecs::{
        component::{ComponentId, HookContext},
        entity::EntityCloner,
        system::QueryLens,
        world::DeferredWorld,
    },
    prelude::*,
};
use core::ops::{Deref, RangeInclusive};
use firewheel::nodes::{
    sampler::{SamplerConfig, SamplerNode, SamplerState},
    volume::VolumeNode,
};
use sample_effects::{EffectOf, SampleEffects};

use crate::{
    SeedlingSystems,
    context::AudioContext,
    edge::{PendingConnections, PendingEdge},
    error::SeedlingError,
    node::{EffectId, FirewheelNode, RegisterNode},
    pool::label::PoolLabelContainer,
    prelude::PoolLabel,
    sample::{OnComplete, PlaybackParams, PlaybackSettings, SamplePlayer},
};

pub mod dynamic;
mod entity_set;
pub mod label;
mod queue;
pub mod sample_effects;

pub(crate) struct SamplePoolPlugin;

impl Plugin for SamplePoolPlugin {
    fn build(&self, app: &mut App) {
        app.register_node::<SamplerNode>()
            .add_systems(
                Last,
                (
                    (populate_pool, queue::grow_pools)
                        .chain()
                        .before(SeedlingSystems::Acquire),
                    (remove_finished, queue::assign_default, retrieve_state)
                        .before(SeedlingSystems::Pool)
                        .after(SeedlingSystems::Connect),
                    watch_sample_players
                        .before(SeedlingSystems::Queue)
                        .after(SeedlingSystems::Pool),
                    (queue::assign_work, queue::update_followers)
                        .chain()
                        .in_set(SeedlingSystems::Pool),
                ),
            )
            .add_plugins(dynamic::DynamicPlugin);
    }
}

#[derive(Debug, Component)]
#[relationship(relationship_target = Samplers)]
pub struct SamplerOf(pub Entity);

#[derive(Debug, Component)]
#[relationship_target(relationship = SamplerOf, linked_spawn)]
pub struct Samplers(Vec<Entity>);

#[derive(Component)]
struct SamplerStateWrapper(SamplerState);

#[derive(Debug, Component)]
#[relationship(relationship_target = SamplerAssignment)]
#[component(on_remove = Self::on_remove_hook)]
pub struct SamplerAssignmentOf(pub Entity);

impl SamplerAssignmentOf {
    fn on_remove_hook(mut world: DeferredWorld, context: HookContext) {
        if let Some(mut sampler) = world.get_mut::<SamplerNode>(context.entity) {
            sampler.stop();
        }
    }
}

#[derive(Debug, Component)]
#[relationship_target(relationship = SamplerAssignmentOf)]
#[component(on_remove = Self::on_remove_hook)]
pub struct SamplerAssignment(Entity);

impl SamplerAssignment {
    fn on_remove_hook(mut world: DeferredWorld, context: HookContext) {
        if let Some(mut player) = world.get_mut::<SamplePlayer>(context.entity) {
            player.clear_sampler();
        }
    }
}

#[derive(Component)]
struct PoolShape(Vec<ComponentId>);

fn fetch_effect_ids(
    effects: &[Entity],
    lens: &mut QueryLens<&EffectId>,
) -> core::result::Result<Vec<ComponentId>, SeedlingError> {
    let query = lens.query();

    let mut effect_ids = Vec::new();
    effect_ids.reserve_exact(effects.len());
    for entity in effects {
        let id = query
            .get(*entity)
            .map_err(|_| SeedlingError::MissingEffect {
                empty_entity: *entity,
            })?;

        effect_ids.push(id.0);
    }

    Ok(effect_ids)
}

fn retrieve_state(
    q: Query<(Entity, &FirewheelNode), (With<SamplerNode>, Without<SamplerStateWrapper>)>,
    mut commands: Commands,
    mut context: ResMut<AudioContext>,
) {
    if q.iter().len() == 0 {
        return;
    }

    context.with(|ctx| {
        for (entity, node_id) in q.iter() {
            let Some(state) = ctx.node_state::<SamplerState>(node_id.0) else {
                continue;
            };
            commands
                .entity(entity)
                .insert(SamplerStateWrapper(state.clone()));
        }
    });
}

/// A kind of specialization of [`ParamFollower`] for
/// sampler nodes.
fn watch_sample_players(
    mut q: Query<(&mut SamplerNode, &SamplerAssignmentOf)>,
    samples: Query<&PlaybackParams>,
) {
    for (mut sampler_node, sample) in q.iter_mut() {
        let Ok(settings) = samples.get(sample.0) else {
            continue;
        };

        sampler_node.playhead = settings.playhead.clone();
        sampler_node.playback = settings.playback.clone();
        sampler_node.speed = settings.speed;
    }
}

fn spawn_chain(
    bus: Entity,
    config: Option<SamplerConfig>,
    effects: &[Entity],
    commands: &mut Commands,
) -> Entity {
    let sampler = commands
        .spawn((
            SamplerNode::default(),
            config.unwrap_or_default(),
            SamplerOf(bus),
        ))
        .id();

    let effects = effects.to_vec();
    commands.queue(move |world: &mut World| -> Result {
        let mut cloner = EntityCloner::build(world);
        cloner.deny::<EffectOf>();
        let mut cloner = cloner.finish();

        let mut chain = Vec::new();
        chain.reserve_exact(effects.len() + 1);
        for effect in effects {
            chain.push(cloner.spawn_clone(world, effect));
        }
        chain.push(bus);

        // Until we come up with a good way to implement the
        // connect trait for `WorldEntityMut`, we're stuck with
        // a bit of boilerplate.
        world
            .get_entity_mut(sampler)?
            .add_children(&chain)
            .entry::<PendingConnections>()
            .or_default()
            .into_mut()
            .push(PendingEdge::new(chain[0], None));

        for pair in chain.windows(2) {
            world
                .get_entity_mut(pair[0])?
                .entry::<PendingConnections>()
                .or_default()
                .into_mut()
                .push(PendingEdge::new(pair[1], None));
        }

        Ok(())
    });

    sampler
}

#[derive(Debug, Clone, Component)]
pub struct PoolSize(pub RangeInclusive<usize>);

#[derive(Debug, Clone, Resource)]
pub struct DefaultPoolSize(pub RangeInclusive<usize>);

fn populate_pool(
    q: Query<
        (
            Entity,
            &SamplerConfig,
            Option<&PoolSize>,
            Option<&SampleEffects>,
            Option<&EffectId>,
        ),
        (
            With<PoolLabelContainer>,
            Without<SamplePlayer>,
            Without<Samplers>,
        ),
    >,
    mut effects: Query<&EffectId>,
    default_pool_size: Res<DefaultPoolSize>,
    mut commands: Commands,
) -> Result {
    for (pool, config, size, pool_effects, effect_id) in &q {
        if effect_id.is_none() {
            commands.entity(pool).insert(VolumeNode::default());
        }

        let component_ids = fetch_effect_ids(
            pool_effects.map(|e| e.deref()).unwrap_or(&[]),
            &mut effects.as_query_lens(),
        )?;

        let size = size
            .map(|p| p.0.clone())
            .unwrap_or(default_pool_size.0.clone());

        commands
            .entity(pool)
            .insert((PoolShape(component_ids), PoolSize(size.clone())));

        let size = (*size.start()).max(1);
        let config = config.clone();
        for _ in 0..size {
            spawn_chain(
                pool,
                Some(config.clone()),
                pool_effects.map(|e| e.deref()).unwrap_or(&[]),
                &mut commands,
            );
        }
    }

    Ok(())
}

/// Automatically remove or despawn sample players when their
/// sample has finished playing.
fn remove_finished(
    nodes: Query<(&SamplerNode, &SamplerAssignmentOf, &SamplerStateWrapper)>,
    samples: Query<(&PlaybackSettings, &PoolLabelContainer)>,
    mut commands: Commands,
) {
    for (node, active, state) in nodes.iter() {
        let finished = state.0.finished() == node.sequence.id();

        // The sample completed playback in one-shot mode.
        if finished {
            let Ok((settings, container)) = samples.get(active.0) else {
                continue;
            };

            match settings.on_complete {
                OnComplete::Preserve => {
                    commands.entity(active.0).remove::<SamplerAssignment>();
                }
                OnComplete::Remove => {
                    commands
                        .entity(active.0)
                        .remove_by_id(container.label_id)
                        .remove_with_requires::<(
                            SampleEffects,
                            SamplePlayer,
                            PoolLabelContainer,
                            SamplerAssignment,
                        )>();
                }
                OnComplete::Despawn => {
                    commands.entity(active.0).despawn();
                }
            }
        }
    }
}

/// A pool despawner command.
///
/// Despawn a sample pool, cleaning up its resources
/// in the ECS and audio graph.
///
/// Despawning the terminal volume node recursively
/// will produce the same effect.
///
/// This can be used directly or via the [`PoolCommands`] trait.
///
/// ```
/// # use bevy_ecs::prelude::*;
/// # use bevy_seedling::prelude::*;
/// #[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
/// struct MyLabel;
///
/// fn system(mut commands: Commands) {
///     commands.queue(PoolDespawn::new(MyLabel));
/// }
/// ```
#[derive(Debug)]
pub struct PoolDespawn<T>(T);

impl<T: PoolLabel + Component> PoolDespawn<T> {
    /// Construct a new [`PoolDespawn`] with the provided label.
    pub fn new(label: T) -> Self {
        Self(label)
    }
}

impl<T: PoolLabel + Component> Command for PoolDespawn<T> {
    fn apply(self, world: &mut World) {
        let mut roots =
            world.query_filtered::<(Entity, &PoolLabelContainer), (With<T>, With<Samplers>, With<FirewheelNode>)>();

        let roots: Vec<_> = roots
            .iter(world)
            .map(|(root, label)| (root, label.clone()))
            .collect();

        let mut commands = world.commands();

        let interned = self.0.intern();
        for (root, label) in roots {
            if label.label == interned {
                commands.entity(root).despawn();
            }
        }
    }
}

/// Provides methods on [`Commands`] to manage sample pools.
pub trait PoolCommands {
    /// Despawn a sample pool, cleaning up its resources
    /// in the ECS and audio graph.
    ///
    /// Despawning the terminal volume node recursively
    /// will produce the same effect.
    fn despawn_pool<T: PoolLabel + Component>(&mut self, label: T);
}

impl PoolCommands for Commands<'_, '_> {
    fn despawn_pool<T: PoolLabel + Component>(&mut self, label: T) {
        self.queue(PoolDespawn::new(label));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        prelude::LowPassNode,
        sample_effects,
        test::{prepare_app, run},
    };
    use bevy_seedling_macros::PoolLabel;

    #[derive(PoolLabel, Clone, Debug, PartialEq, Eq, Hash)]
    struct TestPool;

    #[test]
    fn test_spawn() {
        let mut app = prepare_app(|mut commands: Commands| {
            commands.spawn((TestPool, sample_effects![LowPassNode::default()]));
        });

        run(&mut app, |q: Query<&Samplers, With<TestPool>>| {
            assert_eq!(q.iter().len(), 1);
        });
    }

    #[test]
    fn test_despawn() {
        let mut app = prepare_app(|mut commands: Commands| {
            commands.spawn((
                TestPool,
                PoolSize(4..=32),
                sample_effects![LowPassNode::default()],
            ));
        });

        run(&mut app, |pool_nodes: Query<&FirewheelNode>| {
            // 2 * 4 (sampler and low pass nodes) + (pool volume) + 1 (global volume)
            assert_eq!(pool_nodes.iter().count(), 10);
        });

        run(&mut app, |mut commands: Commands| {
            commands.despawn_pool(TestPool);
        });

        app.update();

        run(&mut app, |pool_nodes: Query<&FirewheelNode>| {
            // 1 (global volume)
            assert_eq!(pool_nodes.iter().count(), 1);
        });
    }

    #[test]
    fn test_playback_starts() {
        let mut app = prepare_app(|mut commands: Commands, server: Res<AssetServer>| {
            commands.spawn((TestPool, sample_effects![LowPassNode::default()]));
            commands.spawn((
                TestPool,
                SamplePlayer::new(server.load("caw.ogg")),
                EmptyComponent,
                PlaybackSettings::LOOP,
            ));
        });

        loop {
            let players = run(
                &mut app,
                |q: Query<Entity, (With<SamplePlayer>, With<SamplerAssignment>)>| q.iter().len(),
            );

            if players == 1 {
                break;
            }

            app.update();
        }
    }

    #[derive(Component)]
    struct EmptyComponent;

    #[test]
    fn test_remove_in_dynamic() {
        let mut app = prepare_app(|mut commands: Commands, server: Res<AssetServer>| {
            // We'll play a short sample
            commands.spawn((
                SamplePlayer::new(server.load("sine_440hz_1ms.wav")),
                EmptyComponent,
                PlaybackSettings::REMOVE,
                sample_effects![LowPassNode::default()],
            ));
        });

        // Then wait until the sample player is removed.
        loop {
            let players = run(
                &mut app,
                |q: Query<Entity, (With<SamplePlayer>, With<EmptyComponent>)>| q.iter().len(),
            );

            if players == 0 {
                break;
            }

            app.update();
        }

        // Once removed, we'll verify that _all_ audio-related components are removed.
        let world = app.world_mut();
        let mut q = world.query_filtered::<EntityRef, With<EmptyComponent>>();
        let entity = q.single(world).unwrap();

        let archetype = entity.archetype();

        assert_eq!(archetype.components().count(), 1);
        assert!(entity.contains::<EmptyComponent>());
    }

    #[test]
    fn test_remove_in_pool() {
        let mut app = prepare_app(|mut commands: Commands, server: Res<AssetServer>| {
            commands.spawn((TestPool, sample_effects![LowPassNode::default()]));

            commands.spawn((
                TestPool,
                SamplePlayer::new(server.load("sine_440hz_1ms.wav")),
                EmptyComponent,
                PlaybackSettings::REMOVE,
            ));
        });

        // Then wait until the sample player is removed.
        loop {
            let players = run(
                &mut app,
                |q: Query<Entity, (With<SamplePlayer>, With<EmptyComponent>)>| q.iter().len(),
            );

            if players == 0 {
                break;
            }

            app.update();
        }

        // Once removed, we'll verify that _all_ audio-related components are removed.
        let world = app.world_mut();
        let mut q = world.query_filtered::<EntityRef, With<EmptyComponent>>();
        let entity = q.single(world).unwrap();

        let archetype = entity.archetype();

        assert_eq!(archetype.components().count(), 1);
        assert!(entity.contains::<EmptyComponent>());
    }
}
