use bevy::{ecs::entity::EntityCloner, prelude::*};
use firewheel::nodes::sampler::{SamplerConfig, SamplerNode};
use sample_effects::{EffectOf, SampleEffects};

use crate::{
    SeedlingSystems,
    edge::{PendingConnections, PendingEdge},
    node::RegisterNode,
    pool::label::PoolLabelContainer,
};

pub mod dynamic;
pub mod sample_effects;

pub(crate) struct SamplePoolPlugin;

impl Plugin for SamplePoolPlugin {
    fn build(&self, app: &mut App) {
        app.register_node::<SamplerNode>()
            .add_systems(Last, populate_pool.before(SeedlingSystems::Acquire))
            .add_plugins(dynamic::DynamicPlugin);
    }
}

#[derive(Debug, Component)]
#[relationship(relationship_target = Samplers)]
pub struct SamplerOf(pub Entity);

#[derive(Debug, Component)]
#[relationship_target(relationship = SamplerOf, linked_spawn)]
pub struct Samplers(Vec<Entity>);

fn spawn_chain(
    bus: Entity,
    config: Option<SamplerConfig>,
    effects: &SampleEffects,
    commands: &mut Commands,
) -> Entity {
    let sampler = commands
        .spawn((
            SamplerNode::default(),
            config.unwrap_or_default(),
            SamplerOf(bus),
        ))
        .id();

    let effects: Vec<_> = effects.iter().collect();
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

fn populate_pool(q: Query<(&PoolLabelContainer), Without<Samplers>>) {
    todo!()
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

// /// Spawn a sampler pool with an initial size.
// #[cfg_attr(debug_assertions, track_caller)]
// fn spawn_pool<'a, L: PoolLabel + Component + Clone>(
//     label: L,
//     size: core::ops::RangeInclusive<usize>,
//     defaults: SamplePoolTypes,
//     commands: &'a mut Commands,
// ) -> EntityCommands<'a> {
//     commands.despawn_pool(label.clone());

//     let bus = commands
//         .spawn((
//             VolumeNode {
//                 volume: Volume::Linear(1.0),
//             },
//             label.clone(),
//             NodeRank::default(),
//             PoolRange(size.clone()),
//         ))
//         .id();

//     let mut nodes = Vec::new();
//     nodes.reserve_exact(*size.start());
//     for _ in 0..*size.start() {
//         let node = spawn_chain(bus, &defaults, label.clone(), commands);
//         nodes.push(node);
//     }

//     bus
// }
