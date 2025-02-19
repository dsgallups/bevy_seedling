use crate::node::NodeConstructor;
use bevy_ecs::prelude::Component;
use firewheel::{channel_config::NonZeroChannelCount, nodes::sampler, nodes::volume};

#[derive(Debug, Component)]
pub struct VolumeConfig {
    pub channels: NonZeroChannelCount,
    pub smooth_secs: f32,
}

impl Default for VolumeConfig {
    fn default() -> Self {
        Self {
            channels: NonZeroChannelCount::new(2).unwrap(),
            smooth_secs: 10.0 / 1_000.0,
        }
    }
}

impl NodeConstructor for volume::VolumeParams {
    type Configuration = VolumeConfig;

    fn construct(
        &self,
        _: &firewheel::core::StreamInfo,
        config: &Self::Configuration,
    ) -> impl firewheel::node::AudioNodeConstructor + 'static {
        self.constructor(
            config.channels,
            volume::VolumeNodeConfig {
                smooth_secs: config.smooth_secs,
            },
        )
    }
}

impl NodeConstructor for sampler::SamplerHandle {
    type Configuration = sampler::SamplerConfig;
    fn construct(
        &self,
        _: &firewheel::core::StreamInfo,
        config: &Self::Configuration,
    ) -> impl firewheel::node::AudioNodeConstructor + 'static {
        self.constructor(Default::default(), *config)
    }
}
