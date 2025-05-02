use std::ops::Deref;

use bevy::{
    ecs::{entity::EntityCloner, relationship::Relationship},
    platform::collections::HashMap,
    prelude::*,
};
use firewheel::nodes::sampler::{RepeatMode, SamplerConfig, SamplerNode};

use crate::{
    node::{EffectId, follower::FollowerOf},
    pool::label::PoolLabelContainer,
    prelude::DefaultPool,
    sample::{PlaybackSettings, QueuedSample, Sample, SamplePlayer},
};

use super::{
    PoolShape, PoolSize, SamplerAssignmentOf, SamplerOf, SamplerStateWrapper, Samplers,
    sample_effects::{EffectOf, SampleEffects},
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
struct SamplerScore {
    is_looping: bool,
    has_assignment: bool,
    raw_score: u64,
}

/// Scan through the set of pending sample players
/// and assign work to the most appropriate sampler node.
pub(super) fn assign_work(
    mut queued_samples: Query<
        (
            Entity,
            &mut SamplePlayer,
            &PlaybackSettings,
            &PoolLabelContainer,
            Option<&SampleEffects>,
        ),
        With<QueuedSample>,
    >,
    pools: Query<(
        Entity,
        &PoolLabelContainer,
        &Samplers,
        &PoolSize,
        &PoolShape,
        Option<&SampleEffects>,
        &SamplerConfig,
    )>,
    mut nodes: Query<
        (
            Entity,
            &mut SamplerNode,
            &SamplerStateWrapper,
            Option<&SamplerAssignmentOf>,
        ),
        With<SamplerOf>,
    >,
    active_samples: Query<&PlaybackSettings>,
    mut effects: Query<&EffectId, With<EffectOf>>,
    assets: Res<Assets<Sample>>,
    mut commands: Commands,
) -> Result {
    let mut queued_samples: HashMap<_, Vec<_>> = queued_samples
        .iter_mut()
        .filter_map(|(entity, player, settings, label, effects)| {
            let asset = assets.get(&player.sample)?;

            Some((label.label, (entity, player, settings, asset, effects)))
        })
        .fold(HashMap::new(), |mut acc, (key, value)| {
            acc.entry(key).or_default().push(value);
            acc
        });

    if queued_samples.is_empty() {
        return Ok(());
    }

    let pools: Vec<_> = pools
        .iter()
        .filter(|(_, label, ..)| queued_samples.contains_key(&label.label))
        .collect();

    for (pool_entity, label, samplers, size, pool_shape, pool_effects, pool_config) in pools {
        let mut queued_samples = queued_samples.remove(&label.label).unwrap();

        // if there is enough sampler availability in the pool,
        // don't bother sorting samples by priority

        let inactive_samplers: Vec<_> = samplers
            .iter()
            .filter(|s| nodes.get(*s).is_ok_and(|n| n.3.is_none()))
            .collect();

        if inactive_samplers.len() >= queued_samples.len() {
            let mut inactive = inactive_samplers.into_iter();

            for (sample_entity, mut player, settings, asset, sample_effects) in queued_samples {
                let (sampler_entity, mut params, state, _) =
                    nodes.get_mut(inactive.next().unwrap())?;

                params.set_sample(asset.get(), settings.volume, settings.repeat_mode);
                player.set_sampler(sampler_entity, state.0.clone());
                state.0.clear_finished();

                // normalize sample effects
                if sample_effects.is_some() && pool_effects.is_none() {
                    match player.sample.path() {
                        Some(path) => warn!(
                            "Queued sample \"{}\" with effects in an effect-less pool.",
                            path
                        ),
                        None => warn!("Queued sample with effects in an effect-less pool."),
                    }
                }

                if let Some(pool_effects) = pool_effects {
                    match sample_effects {
                        Some(sample_effects) => {
                            let component_ids = match super::fetch_effect_ids(
                                sample_effects,
                                &mut effects.as_query_lens(),
                            ) {
                                Ok(ids) => ids,
                                Err(e) => {
                                    error!("{e}");

                                    continue;
                                }
                            };

                            if component_ids != pool_shape.0 {
                                // N will never be large enough for this to be a concern
                                if component_ids.iter().any(|id| !pool_shape.0.contains(id)) {
                                    match player.sample.path() {
                                        Some(path) => warn!(
                                            "Queued sample \"{}\" contains one or more effects that the pool does not.",
                                            path
                                        ),
                                        None => warn!(
                                            "Queued sample contains one or more effects that the pool does not."
                                        ),
                                    }
                                }

                                let mut new_effects = Vec::new();
                                new_effects.reserve_exact(pool_shape.0.len());
                                let mut clone_into = Vec::new();

                                for (effect, id) in pool_effects.iter().zip(&pool_shape.0) {
                                    match component_ids.iter().position(|c| c == id) {
                                        Some(index) => {
                                            new_effects.push(sample_effects[index]);
                                        }
                                        None => {
                                            let empty = commands.spawn_empty().id();

                                            clone_into.push((empty, effect));
                                            new_effects.push(empty);
                                        }
                                    }
                                }

                                commands
                                    .entity(sample_entity)
                                    .remove_related::<EffectOf>(sample_effects)
                                    .add_related::<EffectOf>(&new_effects);

                                commands.queue(move |world: &mut World| {
                                    let mut cloner = EntityCloner::build(world);
                                    cloner.deny::<EffectOf>();
                                    let mut cloner = cloner.finish();

                                    for (dest, src) in clone_into {
                                        cloner.clone_entity(world, src, dest);
                                    }
                                });
                            }
                        }
                        None => {
                            let pool_effects: Vec<_> = pool_effects.iter().collect();
                            commands.queue(move |world: &mut World| {
                                let mut cloner = EntityCloner::build(world);
                                cloner.deny::<EffectOf>();
                                let mut cloner = cloner.finish();

                                let mut sample_effects = Vec::new();
                                sample_effects.reserve_exact(pool_effects.len());
                                for effect in pool_effects {
                                    let sample_effect = cloner.spawn_clone(world, effect);
                                    sample_effects.push(sample_effect);
                                }

                                world
                                    .entity_mut(sample_entity)
                                    .add_related::<EffectOf>(&sample_effects);
                            });
                        }
                    }
                }

                commands
                    .entity(sample_entity)
                    .remove::<QueuedSample>()
                    .add_one_related::<SamplerAssignmentOf>(sampler_entity);
            }

            continue;
        }

        // first, sort the available samplers
        let mut sampler_scores = Vec::new();
        for (sampler_entity, params, state, assignment) in nodes.iter_many(samplers.iter()) {
            let raw_score = state.0.worker_score(params);
            let has_assignment = assignment.is_some();
            let is_looping = assignment
                .and_then(|a| {
                    active_samples
                        .get(a.0)
                        .ok()
                        .map(|s| s.repeat_mode != RepeatMode::PlayOnce)
                })
                .unwrap_or_default();

            sampler_scores.push((
                sampler_entity,
                SamplerScore {
                    raw_score,
                    has_assignment,
                    is_looping,
                },
            ));
        }

        sampler_scores.sort_by_key(|pair| pair.1);

        // then sort the queued samples
        queued_samples.sort_by_key(|s| s.2.repeat_mode == RepeatMode::RepeatEndlessly);

        let difference = queued_samples.len() - sampler_scores.len();

        for (sampler, queued) in sampler_scores.into_iter().zip(queued_samples.into_iter()) {
            let (sample_entity, mut player, settings, asset, sample_effects) = queued;

            let (sampler_entity, mut params, state, _) = nodes.get_mut(sampler.0)?;

            params.set_sample(asset.get(), settings.volume, settings.repeat_mode);
            player.set_sampler(sampler_entity, state.0.clone());
            state.0.clear_finished();

            // normalize sample effects
            if sample_effects.is_some() && pool_effects.is_none() {
                match player.sample.path() {
                    Some(path) => warn!(
                        "Queued sample \"{}\" with effects in an effect-less pool.",
                        path
                    ),
                    None => warn!("Queued sample with effects in an effect-less pool."),
                }
            }

            if let Some(pool_effects) = pool_effects {
                match sample_effects {
                    Some(sample_effects) => {
                        let component_ids = match super::fetch_effect_ids(
                            sample_effects,
                            &mut effects.as_query_lens(),
                        ) {
                            Ok(ids) => ids,
                            Err(e) => {
                                error!("{e}");

                                continue;
                            }
                        };

                        if component_ids != pool_shape.0 {
                            // N will never be large enough for this to be a concern
                            if component_ids.iter().any(|id| !pool_shape.0.contains(id)) {
                                match player.sample.path() {
                                    Some(path) => warn!(
                                        "Queued sample \"{}\" contains one or more effects that the pool does not.",
                                        path
                                    ),
                                    None => warn!(
                                        "Queued sample contains one or more effects that the pool does not."
                                    ),
                                }
                            }

                            let mut new_effects = Vec::new();
                            new_effects.reserve_exact(pool_shape.0.len());
                            let mut clone_into = Vec::new();

                            for (effect, id) in pool_effects.iter().zip(&pool_shape.0) {
                                match component_ids.iter().position(|c| c == id) {
                                    Some(index) => {
                                        new_effects.push(sample_effects[index]);
                                    }
                                    None => {
                                        let empty = commands.spawn_empty().id();

                                        clone_into.push((empty, effect));
                                        new_effects.push(empty);
                                    }
                                }
                            }

                            commands
                                .entity(sample_entity)
                                .remove_related::<EffectOf>(sample_effects)
                                .add_related::<EffectOf>(&new_effects);

                            commands.queue(move |world: &mut World| {
                                let mut cloner = EntityCloner::build(world);
                                cloner.deny::<EffectOf>();
                                let mut cloner = cloner.finish();

                                for (dest, src) in clone_into {
                                    cloner.clone_entity(world, src, dest);
                                }
                            });
                        }
                    }
                    None => {
                        let pool_effects: Vec<_> = pool_effects.iter().collect();
                        commands.queue(move |world: &mut World| {
                            let mut cloner = EntityCloner::build(world);
                            cloner.deny::<EffectOf>();
                            let mut cloner = cloner.finish();

                            let mut sample_effects = Vec::new();
                            sample_effects.reserve_exact(pool_effects.len());
                            for effect in pool_effects {
                                let sample_effect = cloner.spawn_clone(world, effect);
                                sample_effects.push(sample_effect);
                            }

                            world
                                .entity_mut(sample_entity)
                                .add_related::<EffectOf>(&sample_effects);
                        });
                    }
                }
            }

            commands
                .entity(sample_entity)
                .remove::<QueuedSample>()
                .add_one_related::<SamplerAssignmentOf>(sampler_entity);
        }

        // attempt to grow pool if possible
        if samplers.len() < *size.0.end() {
            let growth_size = difference.max(8);
            let new_size = (samplers.len() + growth_size).min(*size.0.end());

            for _ in samplers.len()..new_size {
                super::spawn_chain(
                    pool_entity,
                    Some(pool_config.clone()),
                    pool_effects.map(|e| e.deref()).unwrap_or(&[]),
                    &mut commands,
                );
            }
        }
    }

    Ok(())
}

pub(super) fn update_followers(
    samplers: Query<(&Children, &SamplerAssignmentOf), Changed<SamplerAssignmentOf>>,
    samples: Query<&SampleEffects>,
    mut commands: Commands,
) {
    for (children, assignment) in &samplers {
        let Ok(effects) = samples.get(assignment.get()) else {
            continue;
        };

        for (effect, follower) in effects.iter().zip(children.iter()) {
            commands.entity(follower).insert(FollowerOf(effect));
        }
    }
}

// Stop playback if the source entity no longer exists.
pub(super) fn monitor_active(
    mut nodes: Query<(Entity, &mut SamplerNode, &SamplerAssignmentOf, &Children)>,
    samples: Query<&SamplePlayer>,
    mut commands: Commands,
) {
    for (node_entity, mut sampler, active, effects_chain) in nodes.iter_mut() {
        if samples.get(active.0).is_err() {
            sampler.stop();

            commands.entity(node_entity).remove::<SamplerAssignmentOf>();

            for effect in effects_chain.iter() {
                commands.entity(effect).remove::<FollowerOf>();
            }
        }
    }
}

/// Assign the default pool label to a sample player that has no label.
pub(super) fn assign_default(
    samples: Query<
        Entity,
        (
            With<SamplePlayer>,
            Without<PoolLabelContainer>,
            Without<SampleEffects>,
        ),
    >,
    mut commands: Commands,
) {
    for sample in samples.iter() {
        commands.entity(sample).insert(DefaultPool);
    }
}
