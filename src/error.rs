use bevy::prelude::Entity;
use firewheel::diff::PatchError;

// TODO: add location tracking where relevant
#[derive(Debug)]
pub enum SeedlingError {
    /// An error occurred when applying a Firewheel `Patch`
    /// to an audio node.
    PatchError { ty: &'static str, error: PatchError },
    /// An error occurred when attempting to connect two
    /// audio nodes.
    ConnectionError {
        source: Entity,
        dest: Entity,
        error: String,
    },
    /// A sample effect relationship was spawned with an empty
    /// effect entity.
    MissingEffect { empty_entity: Entity },
}

impl core::fmt::Display for SeedlingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PatchError { ty, error } => {
                write!(f, "Failed to apply audio patch to `{ty}`: {error:?}")
            }
            Self::ConnectionError { error, .. } => {
                write!(f, "Failed to connect audio nodes: {error}")
            }
            Self::MissingEffect { empty_entity } => {
                write!(f, "Expected audio node in `AudioEffect` relationship")
            }
        }
    }
}

impl core::error::Error for SeedlingError {}
