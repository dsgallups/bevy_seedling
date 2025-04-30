use bevy::prelude::*;
use smallvec::SmallVec;

#[derive(Debug, Component)]
#[relationship(relationship_target = SampleEffects)]
pub struct EffectOf(pub Entity);

#[derive(Debug, Component)]
#[relationship_target(relationship = EffectOf, linked_spawn)]
pub struct SampleEffects(SmallVec<[Entity; 2]>);

impl core::ops::Deref for SampleEffects {
    type Target = [Entity];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[doc(hidden)]
pub use bevy::ecs::spawn::Spawn;

#[macro_export]
macro_rules! sample_effects {
    [$($effect:expr),*$(,)?] => {
        <$crate::pool2::sample_effects::SampleEffects>::spawn(($($crate::pool2::sample_effects::Spawn($effect)),*))
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::*;
    use crate::profiling::ProfilingBackend;
    use crate::test::{prepare_app, run};
    use bevy::ecs::system::RunSystemOnce;

    fn test_clone() {
        #[derive(PoolLabel, Debug, Clone, Hash, PartialEq, Eq)]
        struct MyPool;

        let app = prepare_app(|mut commands: Commands, server: Res<AssetServer>| {
            // Spawn a sample pool
            commands.spawn((
                MyPool,
                sample_effects![LowPassNode::default(), SpatialBasicNode::default()],
            ));

            // Spawn a sample with effects
            commands.spawn((
                SamplePlayer::new(server.load("caw.ogg")),
                sample_effects![LowPassNode::default(), SpatialBasicNode::default()],
            ));

            // bsn! {
            //     (
            //         SamplePlayer(@"caw.ogg"),
            //         SampleEffects [ LowPassNode, SpatialBasicNode ],
            //     )
            // }
        });
    }
}
