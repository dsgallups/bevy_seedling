use super::sample_effects::EffectOf;
use crate::{
    error::SeedlingError,
    node::EffectId,
    pool::label::PoolLabelContainer,
    pool2::sample_effects::SampleEffects,
    sample::{QueuedSample, SamplePlayer},
};
use bevy::{ecs::component::ComponentId, platform::collections::HashMap, prelude::*};
use bevy_seedling_macros::PoolLabel;

pub(crate) struct DynamicPlugin;

impl Plugin for DynamicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Registries>()
            .add_systems(PostUpdate, update_auto_pools);
    }
}

/// A label reserved for dynamic pools.
#[derive(PoolLabel, Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct DynamicPoolId(usize);

/// Sets the range for the number dynamic pool sampler nodes.
///
/// When the inner value is `None`, no new dynamic pools will be created.
#[derive(Resource, Clone, Debug)]
pub struct DynamicPoolRange(pub Option<core::ops::RangeInclusive<usize>>);

struct RegistryEntry {
    label: DynamicPoolId,
    pool: Entity,
}

#[derive(Resource, Default)]
struct Registries(HashMap<Vec<ComponentId>, RegistryEntry>);

fn update_auto_pools(
    queued_samples: Query<
        (Entity, &SampleEffects),
        (
            With<QueuedSample>,
            With<SamplePlayer>,
            Without<PoolLabelContainer>,
        ),
    >,
    effects: Query<&EffectId, With<EffectOf>>,
    mut registries: ResMut<Registries>,
    mut commands: Commands,
    dynamic_range: Res<DynamicPoolRange>,
) -> Result {
    let Some(dynamic_range) = dynamic_range.0.clone() else {
        return Ok(());
    };

    for (sample, sample_effects) in queued_samples.iter() {
        let mut component_ids = Vec::new();
        component_ids.reserve_exact(sample_effects.len());

        for effect in sample_effects.iter() {
            let id = effects
                .get(effect)
                .map_err(|_| SeedlingError::MissingEffect {
                    effect_parent: sample,
                    empty_entity: effect,
                })?;
            component_ids.push(id.0);
        }

        match registries.0.get_mut(&component_ids) {
            Some(entry) => {
                commands.entity(sample).insert(entry.label);
            }
            None => {
                let label = DynamicPoolId(registries.0.len());

                // // create the pool
                // super::spawn_pool(
                //     label,
                //     dynamic_range.clone(),
                //     defaults.clone(),
                //     &mut commands,
                // );

                registries.0.insert(
                    component_ids,
                    RegistryEntry {
                        label,
                        pool: todo!(),
                    },
                );

                commands.entity(sample).insert(label);
            }
        }
    }

    Ok(())
}
