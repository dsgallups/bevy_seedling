//! Profiling utilities.

use firewheel::{
    backend::AudioBackend, clock::ClockSeconds, processor::FirewheelProcessor, FirewheelCtx,
    StreamInfo,
};
use std::num::NonZeroU32;

/// A simple audio context that facilitates
/// focused benchmarking.
#[allow(missing_debug_implementations)]
pub struct ProfilingContext {
    pub context: FirewheelCtx<ProfilingBackend>,
    time: ClockSeconds,
    sample_rate_recip: f64,
}

struct ProfilingBackend {
    processor: Option<FirewheelProcessor>,
}

#[derive(Debug)]
struct ProfilingError;

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

    fn available_input_devices() -> Vec<firewheel::backend::DeviceInfo> {
        vec![]
    }

    fn available_output_devices() -> Vec<firewheel::backend::DeviceInfo> {
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

impl ProfilingContext {
    pub fn new(sample_rate: u32) -> Self {
        let mut context = FirewheelCtx::new(Default::default());

        context.start_stream(()).unwrap();

        Self {
            context,
            time: ClockSeconds(0.),
            sample_rate_recip: 1. / sample_rate as f64,
        }
    }

    pub fn process_interleaved(&mut self, input: &[f32], output: &mut [f32]) {
        let samples = output.len() / 2;

        todo!("we need some way of processing the underlying graph directly");
        // self.context.process_interleaved(
        //     input,
        //     output,
        //     2,
        //     2,
        //     samples,
        //     self.time,
        //     StreamStatus::empty(),
        // );

        self.time.0 += self.sample_rate_recip * samples as f64;
    }
}
