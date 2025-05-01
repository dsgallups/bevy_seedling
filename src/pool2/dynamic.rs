use super::{DefaultPoolSize, PoolSize, sample_effects::EffectOf};
use crate::{
    error::SeedlingError,
    node::EffectId,
    pool::label::PoolLabelContainer,
    pool2::sample_effects::SampleEffects,
    sample::{QueuedSample, SamplePlayer},
};
use bevy::{
    ecs::{component::ComponentId, system::QueryLens},
    platform::collections::HashMap,
    prelude::*,
};
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

// /// Sets the range for the number dynamic pool sampler nodes.
// ///
// /// When the inner value is `None`, no new dynamic pools will be created.
// #[derive(Resource, Clone, Debug)]
// pub struct DynamicPoolRange(pub Option<core::ops::RangeInclusive<usize>>);

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
    mut effects: Query<&EffectId>,
    mut registries: ResMut<Registries>,
    mut commands: Commands,
    dynamic_range: Res<DefaultPoolSize>,
) -> Result {
    for (sample, sample_effects) in queued_samples.iter() {
        let component_ids =
            match super::fetch_effect_ids(&sample_effects, &mut effects.as_query_lens()) {
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
                let label = DynamicPoolId(registries.0.len());

                let pool = commands
                    .spawn((label, PoolSize(dynamic_range.0.clone())))
                    .id();

                registries
                    .0
                    .insert(component_ids, RegistryEntry { label, pool });

                commands.entity(sample).insert(label);
            }
        }
    }

    Ok(())
}
