use bevy_ecs::prelude::*;
use bevy_transform::components::GlobalTransform;
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
