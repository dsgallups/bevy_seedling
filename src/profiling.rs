//! Profiling utilities.

use firewheel::{
    backend::{AudioBackend, DeviceInfo},
    processor::FirewheelProcessor,
    StreamInfo,
};
use std::num::NonZeroU32;

/// A very simple backend for testing and profiling.
pub struct ProfilingBackend {
    processor: Option<FirewheelProcessor>,
}

impl core::fmt::Debug for ProfilingBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfilingBackend")
            .field("processor", &())
            .finish()
    }
}

#[derive(Debug)]
#[allow(missing_docs)]
pub struct ProfilingError;

impl core::fmt::Display for ProfilingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <_ as core::fmt::Debug>::fmt(self, f)
    }
}

impl std::error::Error for ProfilingError {}

impl AudioBackend for ProfilingBackend {
    type Config = ();

    type StartStreamError = ProfilingError;
    type StreamError = ProfilingError;

    fn available_input_devices() -> Vec<DeviceInfo> {
        vec![]
    }

    fn available_output_devices() -> Vec<DeviceInfo> {
        vec![]
    }

    fn start_stream(_: Self::Config) -> Result<(Self, StreamInfo), Self::StartStreamError> {
        let sample_rate = NonZeroU32::new(48000).unwrap();

        Ok((
            Self { processor: None },
            StreamInfo {
                sample_rate,
                sample_rate_recip: 1.0 / sample_rate.get() as f64,
                max_block_frames: NonZeroU32::new(128).unwrap(),
                num_stream_in_channels: 0,
                num_stream_out_channels: 2,
                declick_frames: NonZeroU32::new(16).unwrap(),
                input_device_name: None,
                output_device_name: None,
                input_to_output_latency_seconds: 0.0,
            },
        ))
    }

    fn set_processor(&mut self, processor: FirewheelProcessor) {
        self.processor = Some(processor);
    }

    fn poll_status(&mut self) -> Result<(), Self::StreamError> {
        Ok(())
    }
}
