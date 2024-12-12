use crate::{AudioContext, InternedNodeLabel, MainBus, NodeLabel};
use bevy_ecs::prelude::*;
use bevy_log::{error, info, warn};
use bevy_utils::HashMap;
use firewheel::node::NodeID;

#[derive(Debug, Component)]
pub struct Node(pub NodeID);

#[derive(Debug)]
pub enum ConnectTarget {
    Label(InternedNodeLabel),
    Entity(Entity),
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

#[derive(Debug, Default, Component)]
pub struct PendingConnections(Vec<ConnectTarget>);

pub trait ConnectNode {
    /// Queue a connection from this entity to the target.
    ///
    /// The connection is deferred, finalizing in the [SeedlingSystems::Connect] set.
    fn connect_to(&mut self, target: impl Into<ConnectTarget>) -> &mut Self;
}

impl ConnectNode for EntityCommands<'_> {
    fn connect_to(&mut self, target: impl Into<ConnectTarget>) -> &mut Self {
        let target = target.into();

        self.entry::<PendingConnections>()
            .or_default()
            .and_modify(|mut pending| {
                pending.0.push(target);
            });

        self
    }
}

#[derive(Default, Debug, Resource)]
pub struct NodeMap(HashMap<InternedNodeLabel, NodeID>);

impl NodeMap {
    pub fn new(main_bus: NodeID) -> Self {
        Self(HashMap::from([(MainBus.intern(), main_bus)]))
    }
}

impl core::ops::Deref for NodeMap {
    type Target = HashMap<InternedNodeLabel, NodeID>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for NodeMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub(crate) fn process_connections(
    mut conn: Query<(&mut PendingConnections, &Node)>,
    targets: Query<&Node>,
    node_map: Res<NodeMap>,
    mut context: ResMut<AudioContext>,
) {
    context.with(|context| {
        if let Some(graph) = context.graph_mut() {
            for (mut connections, source_node) in conn.iter_mut() {
                connections.0.retain(|connection| {
                    let dest_node = match connection {
                        ConnectTarget::Entity(entity) => {
                            let Ok(dest_node) = targets.get(*entity) else {
                                warn!("no target {entity:?} found for audio node connection");
                                return true;
                            };

                            dest_node.0
                        }
                        ConnectTarget::Label(label) => {
                            let Some(dest_node) = node_map.get(label) else {
                                warn!("no active label found for audio node connection");

                                return true;
                            };

                            *dest_node
                        }
                    };

                    match graph.connect(source_node.0, dest_node, &[(0, 0), (1, 1)], false) {
                        Ok(_) => false,
                        Err(e) => {
                            error!("failed to connect audio node to target: {e}");

                            true
                        }
                    }
                });
            }
        }
    });
}
