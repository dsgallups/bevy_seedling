use crate::{AudioContext, InternedNodeLabel, MainBus, NodeLabel};
use bevy_ecs::prelude::*;
use bevy_log::{error, warn};
use bevy_utils::HashMap;
use firewheel::node::NodeID;

/// A newtype wrapper aound [firewheel::node::NodeID].
///
/// The node is automatically removed from the audio
/// graph when this component is removed.
#[derive(Debug, Clone, Copy)]
pub struct Node(pub NodeID);

impl Component for Node {
    const STORAGE_TYPE: bevy_ecs::component::StorageType = bevy_ecs::component::StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy_ecs::component::ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            let Some(node) = world.get::<Node>(entity).copied() else {
                return;
            };

            let mut removals = world.resource_mut::<PendingRemovals>();
            removals.0.push(node.0);
        });
    }
}

#[derive(Debug, Default, Resource)]
pub struct PendingRemovals(Vec<NodeID>);

#[derive(Debug)]
pub enum ConnectTarget {
    Label(InternedNodeLabel),
    Entity(Entity),
}

#[derive(Debug)]
pub struct PendingConnection {
    target: ConnectTarget,
    ports: Option<Vec<(u32, u32)>>,
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
pub struct PendingConnections(Vec<PendingConnection>);

pub trait ConnectNode {
    /// Queue a connection from this entity to the target.
    ///
    /// By default, this provides a port connection of `[(0, 0), (1, 1)]`.
    /// To provide a specific port mapping, use [ConnectNode::connect_with].
    ///
    /// The connection is deferred, finalizing in the [SeedlingSystems::Connect] set.
    fn connect(&mut self, target: impl Into<ConnectTarget>) -> &mut Self;

    /// Queue a connection from this entity to the target with the provided port mappings.
    ///
    /// The connection is deferred, finalizing in the [SeedlingSystems::Connect] set.
    fn connect_with(&mut self, target: impl Into<ConnectTarget>, ports: &[(u32, u32)])
        -> &mut Self;
}

impl ConnectNode for EntityCommands<'_> {
    fn connect(&mut self, target: impl Into<ConnectTarget>) -> &mut Self {
        let target = target.into();

        self.entry::<PendingConnections>()
            .or_default()
            .and_modify(|mut pending| {
                pending.0.push(PendingConnection {
                    target,
                    ports: None,
                });
            });

        self
    }

    fn connect_with(
        &mut self,
        target: impl Into<ConnectTarget>,
        ports: &[(u32, u32)],
    ) -> &mut Self {
        let target = target.into();
        let ports = ports.to_vec();

        self.entry::<PendingConnections>()
            .or_default()
            .and_modify(|mut pending| {
                pending.0.push(PendingConnection {
                    target,
                    ports: Some(ports),
                });
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

/// Automatically connect nodes without manual connections to the main bus.
pub(crate) fn auto_connect(
    nodes: Query<Entity, (With<Node>, Without<PendingConnections>)>,
    mut commands: Commands,
) {
    for node in nodes.iter() {
        commands.entity(node).connect(MainBus);
    }
}

pub(crate) fn process_removals(
    mut removals: ResMut<PendingRemovals>,
    mut context: ResMut<AudioContext>,
) {
    context.with(|context| {
        if let Some(graph) = context.graph_mut() {
            for node in removals.0.drain(..) {
                if graph.remove_node(node).is_err() {
                    error!("attempted to remove non-existent or invalid node from audio graph");
                }
            }
        }
    });
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
                    let dest_node = match connection.target {
                        ConnectTarget::Entity(entity) => {
                            let Ok(dest_node) = targets.get(entity) else {
                                warn!("no target {entity:?} found for audio node connection");
                                return true;
                            };

                            dest_node.0
                        }
                        ConnectTarget::Label(label) => {
                            let Some(dest_node) = node_map.get(&label) else {
                                warn!("no active label found for audio node connection");

                                return true;
                            };

                            *dest_node
                        }
                    };

                    let ports = connection.ports.as_deref().unwrap_or(&[(0, 0), (1, 1)]);

                    match graph.connect(source_node.0, dest_node, ports, false) {
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
