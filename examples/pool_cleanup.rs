//! This example demonstrates how to
//! create and remove a custom pool.

use bevy::{log::LogPlugin, prelude::*};
use bevy_seedling::prelude::*;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            LogPlugin::default(),
            AssetPlugin::default(),
            SeedlingPlugin::default(),
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, remove_pool)
        .run();
}

#[derive(PoolLabel, PartialEq, Eq, Debug, Hash, Clone)]
struct AmbiencePool;

fn startup(server: Res<AssetServer>, mut commands: Commands) {
    // Here we spawn our custom ambience pool.
    commands.spawn(SamplerPool(AmbiencePool));

    // And we start playing our sample in the pool.
    commands.spawn((
        AmbiencePool,
        SamplePlayer::new(server.load("crow_ambience.ogg")),
        PlaybackSettings::LOOP,
    ));

    // Then, we queue up the pool's removal.
    commands.spawn(PoolRemover(Timer::new(
        Duration::from_secs(3),
        TimerMode::Once,
    )));
}

#[derive(Component)]
struct PoolRemover(Timer);

fn remove_pool(mut q: Query<(Entity, &mut PoolRemover)>, time: Res<Time>, mut commands: Commands) {
    for (e, mut remover) in q.iter_mut() {
        remover.0.tick(time.delta());

        if remover.0.just_finished() {
            info!("Cleaning up pool...");

            // This will remove the sampler and volume nodes
            // associated with this pool in both the ECS
            // and audio graph.
            commands.despawn_pool(AmbiencePool);

            commands.entity(e).despawn();
        }
    }
}
