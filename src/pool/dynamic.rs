use super::{DefaultPoolSize, PoolSize, SamplerPool, sample_effects::EffectOf};
use crate::{
    node::EffectId,
    pool::{label::PoolLabelContainer, sample_effects::SampleEffects},
    sample::{QueuedSample, SamplePlayer},
};
use bevy::{
    ecs::{component::ComponentId, entity::EntityCloner},
    platform::collections::HashMap,
    prelude::*,
};
use bevy_seedling_macros::PoolLabel;

pub(super) struct DynamicPlugin;

impl Plugin for DynamicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Registries>()
            .add_systems(PostUpdate, update_dynamic_pools);
    }
}

/// A label reserved for dynamic pools.
#[derive(PoolLabel, Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct DynamicPoolLabel(usize);

struct RegistryEntry {
    label: DynamicPoolLabel,
}

#[derive(Resource, Default)]
struct Registries(HashMap<Vec<ComponentId>, RegistryEntry>);

fn update_dynamic_pools(
    queued_samples: Query<
        (Entity, &SampleEffects),
        (
            With<QueuedSample>,
            With<SamplePlayer>,
            Without<PoolLabelContainer>,
        ),
    >,
    mut effects: Query<&EffectId>,
    mut registries: ResMut<Registries>,
    mut commands: Commands,
    dynamic_range: Res<DefaultPoolSize>,
) -> Result {
    if *dynamic_range.0.end() == 0 {
        return Ok(());
    }

    for (sample, sample_effects) in queued_samples.iter() {
        let component_ids =
            match super::fetch_effect_ids(sample_effects, &mut effects.as_query_lens()) {
                Ok(ids) => ids,
                Err(e) => {
                    error!("{e}");

                    continue;
                }
            };

        match registries.0.get_mut(&component_ids) {
            Some(entry) => {
                commands.entity(sample).insert(entry.label);
            }
            None => {
                let label = DynamicPoolLabel(registries.0.len());

                let bus = commands
                    .spawn((SamplerPool(label), PoolSize(dynamic_range.0.clone())))
                    .id();

                let effects: Vec<_> = sample_effects.iter().collect();
                commands.queue(move |world: &mut World| {
                    let mut cloner = EntityCloner::build(world);
                    cloner.deny::<EffectOf>();
                    let mut cloner = cloner.finish();

                    let mut cloned = Vec::new();
                    for effect in effects {
                        let effect = cloner.spawn_clone(world, effect);
                        cloned.push(effect);
                    }

                    world.entity_mut(bus).add_related::<EffectOf>(&cloned);
                });

                registries.0.insert(component_ids, RegistryEntry { label });

                commands.entity(sample).insert(label);
            }
        }
    }

    Ok(())
}
