//! Node connection and disconnection utilities.

use crate::node::label::InternedNodeLabel;
use crate::prelude::{FirewheelNode, MainBus, NodeLabel};
use bevy_ecs::prelude::*;
use bevy_utils::HashMap;
use firewheel::node::NodeID;

#[cfg(debug_assertions)]
use core::panic::Location;

#[allow(clippy::module_inception)]
mod connect;
mod disconnect;

pub use connect::*;
pub use disconnect::*;

/// A target for node connections.
///
/// [`ConnectTarget`] can be constructed manually or
/// used as a part of the [`Connect`] and [`Disconnect`] APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectTarget {
    /// A global label such as [`MainBus`].
    Label(InternedNodeLabel),
    /// An audio entity.
    Entity(Entity),
    /// An existing node from the audio graph.
    Node(NodeID),
}

/// A pending connection between two nodes.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PendingConnection {
    /// The connection target.
    ///
    /// The connection will be made between this entity's output
    /// and the target's input.
    pub target: ConnectTarget,

    /// An optional [`firewheel`] port mapping.
    ///
    /// The first tuple element represents the source output,
    /// and the second tuple element represents the sink input.
    ///
    /// If an explicit port mapping is not provided,
    /// `[(0, 0), (1, 1)]` is used.
    pub ports: Option<Vec<(u32, u32)>>,

    #[cfg(debug_assertions)]
    pub(crate) origin: &'static Location<'static>,
}

impl PendingConnection {
    /// Construct a new [`PendingConnection`].
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn new(target: impl Into<ConnectTarget>, ports: Option<Vec<(u32, u32)>>) -> Self {
        Self {
            target: target.into(),
            ports,
            #[cfg(debug_assertions)]
            origin: Location::caller(),
        }
    }

    /// An internal constructor for passing context through closures.
    fn new_with_location(
        target: impl Into<ConnectTarget>,
        ports: Option<Vec<(u32, u32)>>,
        #[cfg(debug_assertions)] location: &'static Location<'static>,
    ) -> Self {
        Self {
            target: target.into(),
            ports,
            #[cfg(debug_assertions)]
            origin: location,
        }
    }
}

impl From<NodeID> for ConnectTarget {
    fn from(value: NodeID) -> Self {
        Self::Node(value)
    }
}

impl<T> From<T> for ConnectTarget
where
    T: NodeLabel,
{
    fn from(value: T) -> Self {
        Self::Label(value.intern())
    }
}

impl From<Entity> for ConnectTarget {
    fn from(value: Entity) -> Self {
        Self::Entity(value)
    }
}

const DEFAULT_CONNECTION: &[(u32, u32)] = &[(0, 0), (1, 1)];

/// A map that associates [`NodeLabel`]s with audio
/// graph nodes.
///
/// This will be automatically synchronized for
/// entities with both a [`FirewheelNode`] and [`NodeLabel`]
/// component.
#[derive(Default, Debug, Resource)]
pub struct NodeMap(HashMap<InternedNodeLabel, Entity>);

impl core::ops::Deref for NodeMap {
    type Target = HashMap<InternedNodeLabel, Entity>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for NodeMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Automatically connect nodes without manual connections to the main bus.
pub(crate) fn auto_connect(
    nodes: Query<Entity, (With<FirewheelNode>, Without<PendingConnections>)>,
    mut commands: Commands,
) {
    for node in nodes.iter() {
        commands.entity(node).connect(MainBus);
    }
}
