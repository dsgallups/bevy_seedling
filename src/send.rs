//! One-pole, low-pass filter.

use crate::{
    connect::{ConnectTarget, PendingConnection, PendingConnections},
    prelude::MainBus,
};
use bevy_ecs::prelude::*;
use firewheel::{
    channel_config::{ChannelConfig, ChannelCount, NonZeroChannelCount},
    diff::{Diff, Patch},
    event::NodeEventList,
    node::{
        AudioNode, AudioNodeInfo, AudioNodeProcessor, ConstructProcessorContext, ProcBuffers,
        ProcInfo, ProcessStatus,
    },
    Volume,
};

/// A one-pole, low-pass filter.
#[derive(Diff, Patch, Debug, Clone, Component)]
pub struct SendNode {
    /// The cutoff frequency in hertz.
    pub send_volume: Volume,

    #[diff(skip)]
    pub(crate) target: ConnectTarget,
}

pub(crate) fn connect_sends(
    mut sends: Query<
        (
            Entity,
            &SendNode,
            &SendConfig,
            Option<&mut PendingConnections>,
        ),
        Added<SendNode>,
    >,
    mut commands: Commands,
) {
    for (entity, send_node, send_config, pending) in sends.iter_mut() {
        let target = send_node.target.clone();

        let total_channels = send_config.channels.get().get();
        let ports = (0..total_channels)
            .map(|c| (c + total_channels, c))
            .collect();

        let pending_connection = PendingConnection::new(target, Some(ports));

        match pending {
            Some(mut pending) => {
                pending.push(pending_connection);
            }

            None => {
                let mut pending = PendingConnections::default();
                pending.push(pending_connection);

                let default_ports = (0..total_channels).map(|c| (c, c)).collect();

                pending.push(PendingConnection::new(MainBus, Some(default_ports)));
                commands.entity(entity).insert(pending);
            }
        }
    }
}

impl SendNode {
    pub fn new(send_volume: Volume, target: impl Into<ConnectTarget>) -> Self {
        Self {
            send_volume,
            target: target.into(),
        }
    }
}

#[derive(Debug, Component, Clone)]
pub struct SendConfig {
    pub channels: NonZeroChannelCount,
}

impl Default for SendConfig {
    fn default() -> Self {
        Self {
            channels: NonZeroChannelCount::STEREO,
        }
    }
}

impl AudioNode for SendNode {
    type Configuration = SendConfig;

    fn info(&self, config: &Self::Configuration) -> AudioNodeInfo {
        AudioNodeInfo::new()
            .debug_name("low-pass filter")
            .channel_config(ChannelConfig {
                num_inputs: config.channels.get(),
                // TODO: remove panic
                num_outputs: ChannelCount::new(config.channels.get().get() * 2)
                    .expect("send channel count must not exceed 32"),
            })
            .uses_events(true)
    }

    fn construct_processor(
        &self,
        _: &Self::Configuration,
        _: ConstructProcessorContext,
    ) -> impl AudioNodeProcessor {
        SendProcessor {
            gain: self.send_volume.amp(),
            params: self.clone(),
        }
    }
}

struct SendProcessor {
    params: SendNode,
    gain: f32,
}

impl AudioNodeProcessor for SendProcessor {
    fn process(
        &mut self,
        ProcBuffers {
            inputs, outputs, ..
        }: ProcBuffers,
        proc_info: &ProcInfo,
        events: NodeEventList,
    ) -> ProcessStatus {
        if self.params.patch_list(events) {
            self.gain = self.params.send_volume.amp();
        }

        if proc_info.in_silence_mask.all_channels_silent(inputs.len()) {
            return ProcessStatus::ClearAllOutputs;
        }

        for frame in 0..proc_info.frames {
            for (i, input) in inputs.iter().enumerate() {
                outputs[i][frame] = input[frame];
                outputs[i + inputs.len()][frame] = input[frame] * self.gain;
            }
        }

        ProcessStatus::outputs_not_silent()
    }
}
