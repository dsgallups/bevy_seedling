//! Audio sample node.

use crate::node::Events;
use bevy_asset::{Asset, AssetLoader, Assets, Handle};
use bevy_ecs::prelude::*;
use bevy_reflect::TypePath;
use firewheel::node::AudioNode;
use firewheel::sample_resource::SampleResource;
use firewheel::sampler::one_shot::OneShotSamplerNode;
use std::sync::Arc;

/// An audio sample.
#[derive(Asset, TypePath, Clone)]
pub struct Sample(Arc<dyn SampleResource>);

impl Sample {
    /// Share the inner value.
    pub fn get(&self) -> Arc<dyn SampleResource> {
        self.0.clone()
    }
}

impl core::fmt::Debug for Sample {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Sample").finish_non_exhaustive()
    }
}

#[derive(Debug, Component, Clone)]
pub struct SamplePlayer(pub(crate) Handle<Sample>);

impl From<SamplePlayer> for Box<dyn AudioNode> {
    fn from(_: SamplePlayer) -> Self {
        OneShotSamplerNode::new(Default::default()).into()
    }
}

impl SamplePlayer {
    pub fn new(handle: Handle<Sample>) -> Self {
        Self(handle)
    }
}

/// A simple loader for audio samples.
#[derive(Debug)]
pub struct SampleLoader {
    /// The sampling rate of the audio engine.
    ///
    /// This must be kept in sync with the engine if
    /// the sample rate changes.
    pub sample_rate: u32,
}

/// Errors produced while loading samples.
#[derive(Debug)]
pub enum SampleLoaderError {
    StdIo(std::io::Error),
    Symphonium(String),
}

impl From<std::io::Error> for SampleLoaderError {
    fn from(value: std::io::Error) -> Self {
        Self::StdIo(value)
    }
}

impl From<symphonium::error::LoadError> for SampleLoaderError {
    fn from(value: symphonium::error::LoadError) -> Self {
        Self::Symphonium(value.to_string())
    }
}

impl std::error::Error for SampleLoaderError {}

impl std::fmt::Display for SampleLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StdIo(stdio) => stdio.fmt(f),
            Self::Symphonium(sy) => f.write_str(sy),
        }
    }
}

impl AssetLoader for SampleLoader {
    type Asset = Sample;
    type Settings = ();
    type Error = SampleLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy_asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut bevy_asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        // Unfortunately, we need to bridge the gap between sync and async APIs here.
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let mut hint = symphonia::core::probe::Hint::new();
        hint.with_extension(&load_context.path().to_string_lossy());

        let mut loader = symphonium::SymphoniumLoader::new();
        let source = firewheel::load_audio_file_from_source(
            &mut loader,
            Box::new(std::io::Cursor::new(bytes)),
            Some(hint),
            self.sample_rate,
            Default::default(),
        )?;

        Ok(Sample(Arc::new(source)))
    }

    fn extensions(&self) -> &[&str] {
        &["wav"]
    }
}

/// A marker struct for entities that are waiting
/// for a sample to load.
#[derive(Debug, Component)]
#[component(storage = "SparseSet")]
pub struct LoadingSample;

pub(crate) fn on_add(
    q: Query<Entity, (Added<SamplePlayer>, Without<LoadingSample>)>,
    mut commands: Commands,
) {
    for player in q.iter() {
        commands.entity(player).insert(LoadingSample);
    }
}

pub(crate) fn trigger_pending_samples(
    mut q: Query<(Entity, &SamplePlayer, &mut Events), With<LoadingSample>>,
    mut commands: Commands,
    assets: Res<Assets<Sample>>,
) {
    for (entity, player, mut events) in q.iter_mut() {
        if let Some(asset) = assets.get(&player.0) {
            events.push_custom(firewheel::sampler::one_shot::Sample {
                sample: asset.get(),
                normalized_volume: 1.0,
                stop_other_voices: false,
            });

            commands.entity(entity).remove::<LoadingSample>();
        }
    }
}
