//! Spatial audio components.
//!
//! To enable spatial audio, three conditions are required:
//!
//! 1. The spatial audio node, [`SpatialBasicNode`], must have
//!    a transform.
//! 2. The spatial listener entity must have a [`SpatialListener2D`]
//!    or [`SpatialListener3D`].
//! 3. The spatial listener entity must have a transform.
//!
//! There should only be one [`SpatialListener2D`] or [`SpatialListener3D`]
//! present in the world at a time. If more than one is present, spatial
//! audio will not be calculated.
//!
//! Typically, you'll want to include a [`SpatialBasicNode`] as an effect.
//!
//! ```
//! # use bevy_seedling::prelude::*;
//! # use bevy::prelude::*;
//! fn spawn_spatial(mut commands: Commands, server: Res<AssetServer>) {
//!     // Spawn a player with a transform (1).
//!     commands
//!         .spawn((
//!             SamplePlayer::new(server.load("my_sample.wav")),
//!             Transform::default(),
//!         ))
//!         .effect(SpatialBasicNode::default());
//!
//!     // Then, spawn a listener (2), which automatically inserts
//!     // a transform if it doesn't already exist (3).
//!     commands.spawn(SpatialListener2D);
//! }
//! ```

use bevy_ecs::prelude::*;
use bevy_transform::components::{GlobalTransform, Transform};
use firewheel::nodes::spatial_basic::SpatialBasicNode;

/// A 2D spatial listener.
///
/// When this component is added to an entity with a transform,
/// this transform is used to calculate spatial offsets for all
/// emitters. An emitter is an entity with [`SpatialBasicNode`]
/// and transform components.
///
/// Only a single listener is supported at a time.
/// Multiple listeners will overwrite each other
/// in a non-deterministic order.
#[derive(Debug, Component)]
#[require(Transform)]
pub struct SpatialListener2D;

/// A 3D spatial listener.
///
/// When this component is added to an entity with a transform,
/// this transform is used to calculate spatial offsets for all
/// emitters. An emitter is an entity with [`SpatialBasicNode`]
/// and transform components.
///
/// Only a single listener is supported at a time.
/// Multiple listeners will overwrite each other
/// in a non-deterministic order.
#[derive(Debug, Component)]
#[require(Transform)]
pub struct SpatialListener3D;

pub(crate) fn update_2d_emitters(
    listener: Query<&GlobalTransform, With<SpatialListener2D>>,
    mut emitters: Query<(&mut SpatialBasicNode, &GlobalTransform)>,
) {
    let Ok(listener) = listener.get_single() else {
        return;
    };

    let listener = listener.translation();

    for (mut spatial, transform) in emitters.iter_mut() {
        let translation = transform.translation();

        let x_diff = listener.x - translation.x;
        let y_diff = listener.y - translation.y;

        spatial.offset.x = x_diff;
        spatial.offset.z = y_diff;
    }
}

pub(crate) fn update_3d_emitters(
    listener: Query<&GlobalTransform, With<SpatialListener3D>>,
    mut emitters: Query<(&mut SpatialBasicNode, &GlobalTransform)>,
) {
    let Ok(listener) = listener.get_single() else {
        return;
    };

    let listener = listener.translation();

    for (mut spatial, transform) in emitters.iter_mut() {
        let translation = transform.translation();

        spatial.offset = listener - translation;
    }
}
