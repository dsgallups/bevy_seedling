use bevy::{platform::collections::HashMap, prelude::*};
use firewheel::nodes::sampler::SamplerNode;

use crate::{
    node::follower::FollowerOf,
    pool::label::PoolLabelContainer,
    prelude::DefaultPool,
    sample::{PlaybackSettings, QueuedSample, Sample, SamplePlayer},
};

use super::{
    ActiveSample, PoolSize, SamplerOf, SamplerStateWrapper, Samplers, sample_effects::SampleEffects,
};

/// Scan through the set of pending sample players
/// and assign work to the most appropriate sampler node.
pub(super) fn assign_work(
    mut queued_samples: Query<
        (
            Entity,
            &mut SamplePlayer,
            &PlaybackSettings,
            &PoolLabelContainer,
        ),
        With<QueuedSample>,
    >,
    pools: Query<(
        Entity,
        &PoolLabelContainer,
        &Samplers,
        &PoolSize,
        Option<&SampleEffects>,
    )>,
    mut nodes: Query<(Entity, &mut SamplerNode, &SamplerStateWrapper), With<SamplerOf>>,
    assets: Res<Assets<Sample>>,
    mut commands: Commands,
) {
    let queued_samples: HashMap<_, Vec<_>> = queued_samples
        .iter_mut()
        .filter_map(|(entity, player, settings, label)| {
            let asset = assets.get(&player.sample)?;

            Some((label.label, (entity, player, settings, asset)))
        })
        .fold(HashMap::new(), |mut acc, (key, value)| {
            acc.entry(key).or_default().push(value);
            acc
        });

    if queued_samples.is_empty() {
        return;
    }

    let pools = pools
        .iter()
        .filter(|(_, label, ..)| queued_samples.contains_key(&label.label));

    for (pool_entity, label, samplers, size, effects) in pools {
        todo!()
    }

    // for (sample, mut player, settings, label) in queued_samples.iter_mut() {
    //     let Some(asset) = assets.get(&player.sample) else {
    //         continue;
    //     };

    //     let Some((pool_entity, mut rank, defaults, pool_label, _, pool_range, mut pool_nodes)) =
    //         pools.iter_mut().find(|pool| pool.4.label == label.label)
    //     else {
    //         continue;
    //     };

    //     // try to find the best non-looping candidate
    //     let Some((node_index, node_entity)) = rank
    //         .0
    //         .iter()
    //         .enumerate()
    //         .find_map(|(i, r)| (!r.2).then_some((i, r.0)))
    //     else {
    //         // Try to grow the pool if it's reached max capacity.
    //         // TODO: find a decent way to do this eagerly.
    //         let current_size = pool_nodes.len();
    //         let max_size = *pool_range.0.end();

    //         if current_size < max_size {
    //             let new_size = (current_size * 2).min(max_size);

    //             for _ in 0..new_size - current_size {
    //                 let new_sampler =
    //                     spawn_chain(pool_entity, defaults, pool_label.clone(), &mut commands);
    //                 pool_nodes.0.push(new_sampler);
    //             }
    //         }

    //         continue;
    //     };

    //     let Ok((node_entity, mut params, effects_chain, state)) = nodes.get_mut(node_entity) else {
    //         continue;
    //     };

    //     params.set_sample(asset.get(), settings.volume, settings.repeat_mode);
    //     player.set_sampler(node_entity, state.0.clone());
    //     state.0.clear_finished();

    //     // redirect all parameters to follow the sample source
    //     for effect in effects_chain.0.iter() {
    //         commands.entity(*effect).insert(FollowerOf(sample));
    //     }

    //     // Insert default pool parameters if not present.
    //     let mut entity_commands = commands.entity(sample);
    //     for ty in defaults.0.iter() {
    //         ty.insert_default(&mut entity_commands);
    //     }

    //     entity_commands.remove::<QueuedSample>();

    //     rank.0.remove(node_index);
    //     commands.entity(node_entity).insert(ActiveSample {
    //         sample_entity: sample,
    //     });
    // }
}

// Stop playback if the source entity no longer exists.
pub(super) fn monitor_active(
    mut nodes: Query<(Entity, &mut SamplerNode, &ActiveSample, &Children)>,
    samples: Query<&SamplePlayer>,
    mut commands: Commands,
) {
    for (node_entity, mut sampler, active, effects_chain) in nodes.iter_mut() {
        if samples.get(active.sample_entity).is_err() {
            sampler.stop();

            commands.entity(node_entity).remove::<ActiveSample>();

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
