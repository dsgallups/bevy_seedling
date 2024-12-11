use bevy_asset::{Asset, AssetLoader, Handle};
use bevy_ecs::prelude::*;
use bevy_reflect::TypePath;
use firewheel::sample_resource::SampleResource;
use std::sync::Arc;

#[derive(Asset, TypePath, Clone)]
pub struct Sample(Arc<dyn SampleResource>);

impl Sample {
    pub fn get(&self) -> Arc<dyn SampleResource> {
        self.0.clone()
    }
}

#[derive(Component)]
pub struct SamplePlayer(pub(crate) Handle<Sample>);

impl SamplePlayer {
    pub fn new(handle: Handle<Sample>) -> Self {
        Self(handle)
    }
}

pub struct SampleLoader {
    pub sample_rate: u32,
}

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
