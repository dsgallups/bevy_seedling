use bevy::{ecs::entity::EntityCloner, prelude::*};
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
    node::{EffectId, FirewheelNode, RegisterNode},
    pool::label::PoolLabelContainer,
    sample::PlaybackParams,
};

pub mod dynamic;
mod queue;
pub mod sample_effects;

pub(crate) struct SamplePoolPlugin;

impl Plugin for SamplePoolPlugin {
    fn build(&self, app: &mut App) {
        app.register_node::<SamplerNode>()
            .add_systems(
                Last,
                (
                    populate_pool.before(SeedlingSystems::Acquire),
                    (queue::assign_default, retrieve_state)
                        .before(SeedlingSystems::Pool)
                        .after(SeedlingSystems::Connect),
                    (watch_sample_players, queue::monitor_active)
                        .chain()
                        .before(SeedlingSystems::Queue)
                        .after(SeedlingSystems::Pool),
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

#[derive(Component, Clone, Copy)]
struct ActiveSample {
    sample_entity: Entity,
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
    mut q: Query<(&mut SamplerNode, &ActiveSample)>,
    samples: Query<&PlaybackParams>,
) {
    for (mut sampler_node, sample) in q.iter_mut() {
        let Ok(settings) = samples.get(sample.sample_entity) else {
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
        (With<PoolLabelContainer>, Without<Samplers>),
    >,
    default_pool_size: Res<DefaultPoolSize>,
    mut commands: Commands,
) {
    for (pool, config, size, effects, effect_id) in &q {
        if effect_id.is_none() {
            commands.entity(pool).insert(VolumeNode::default());
        }

        let size = size
            .map(|p| p.0.clone())
            .unwrap_or(default_pool_size.0.clone());

        let size = (*size.start()).max(1);
        let config = config.clone();
        for _ in 0..size {
            spawn_chain(
                pool,
                Some(config.clone()),
                effects.map(|e| e.deref()).unwrap_or(&[]),
                &mut commands,
            );
        }
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

    #[test]
    fn test_spawn() {
        #[derive(Debug, PoolLabel, Clone, PartialEq, Eq, Hash)]
        struct MyLabel;

        let mut app = prepare_app(|mut commands: Commands| {
            commands.spawn((MyLabel, sample_effects![LowPassNode::default()]));
        });

        run(&mut app, |q: Query<&Samplers, With<MyLabel>>| {
            assert_eq!(q.iter().len(), 1);
        });
    }
}
