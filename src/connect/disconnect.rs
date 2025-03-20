use super::{ConnectTarget, NodeMap, PendingConnection, DEFAULT_CONNECTION};
use crate::{context::AudioContext, node::FirewheelNode};
use bevy_ecs::prelude::*;
use bevy_log::error_once;

#[cfg(debug_assertions)]
use core::panic::Location;

/// The set of all pending disconnections for an entity.
///
/// These disconnections are drained and synchronized with the
/// audio graph in the [`SeedlingSystems::Connect`][crate::SeedlingSystems::Connect]
/// set.
#[derive(Debug, Default, Component)]
pub struct PendingDisconnections(Vec<PendingConnection>);

impl PendingDisconnections {
    /// Push a new pending disconnection.
    pub fn push(&mut self, disconnection: PendingConnection) {
        self.0.push(disconnection)
    }
}

/// An [`EntityCommands`] extension trait for disconnecting node entities.
///
/// These methods provide only source -> sink disconnections. The source
/// is the receiver and the sink is the provided target.
///
/// [`EntityCommands`]: bevy_ecs::prelude::EntityCommands
pub trait Disconnect<'a>: Sized {
    /// Queue a disconnection from this entity to the target.
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_seedling::prelude::*;
    /// # fn system(mut commands: Commands) {
    /// // For any node connection...
    /// let node = commands
    ///     .spawn(VolumeNode {
    ///         volume: Volume::Linear(0.5),
    ///     })
    ///     .connect(MainBus)
    ///     .head();
    ///
    /// // We can process a corresponding disconnection.
    /// commands.entity(node).disconnect(MainBus);
    /// # }
    /// ```
    ///
    /// By default, this provides a port disconnection of `[(0, 0), (1, 1)]`,
    /// which represents a simple stereo disconnection.
    /// To provide a specific port mapping, use [`connect_with`][Disconnect::disconnect_with].
    ///
    /// The disconnection is deferred, finalizing in the
    /// [`SeedlingSystems::Connect`][crate::SeedlingSystems::Connect] set.
    #[cfg_attr(debug_assertions, track_caller)]
    fn disconnect(self, target: impl Into<ConnectTarget>) -> DisconnectCommands<'a> {
        self.disconnect_with(target, DEFAULT_CONNECTION)
    }

    /// Queue a disconnection from this entity to the target with the provided port mappings.
    ///
    /// The disconnection is deferred, finalizing in the
    /// [`SeedlingSystems::Connect`][crate::SeedlingSystems::Connect] set.
    #[cfg_attr(debug_assertions, track_caller)]
    fn disconnect_with(
        self,
        target: impl Into<ConnectTarget>,
        ports: &[(u32, u32)],
    ) -> DisconnectCommands<'a>;
}

impl<'a> Disconnect<'a> for EntityCommands<'a> {
    fn disconnect_with(
        mut self,
        target: impl Into<ConnectTarget>,
        ports: &[(u32, u32)],
    ) -> DisconnectCommands<'a> {
        let target = target.into();
        let ports = ports.to_vec();

        #[cfg(debug_assertions)]
        let location = Location::caller();

        self.entry::<PendingDisconnections>()
            .or_default()
            .and_modify(|mut pending| {
                pending.push(PendingConnection::new_with_location(
                    target,
                    Some(ports),
                    #[cfg(debug_assertions)]
                    location,
                ));
            });

        DisconnectCommands::new(self)
    }
}

impl<'a> Disconnect<'a> for DisconnectCommands<'a> {
    #[cfg_attr(debug_assertions, track_caller)]
    fn disconnect_with(
        mut self,
        target: impl Into<ConnectTarget>,
        ports: &[(u32, u32)],
    ) -> DisconnectCommands<'a> {
        let tail = self.head;

        let mut commands = self.commands.commands();
        let mut commands = commands.entity(tail);

        let target = target.into();
        let ports = ports.to_vec();

        #[cfg(debug_assertions)]
        let location = Location::caller();

        commands
            .entry::<PendingDisconnections>()
            .or_default()
            .and_modify(|mut pending| {
                pending.push(PendingConnection::new_with_location(
                    target,
                    Some(ports),
                    #[cfg(debug_assertions)]
                    location,
                ));
            });

        self
    }
}

/// A set of commands for disconnecting nodes.
pub struct DisconnectCommands<'a> {
    commands: EntityCommands<'a>,
    head: Entity,
}

impl<'a> DisconnectCommands<'a> {
    pub(crate) fn new(commands: EntityCommands<'a>) -> Self {
        Self {
            head: commands.id(),
            commands,
        }
    }
}

impl core::fmt::Debug for DisconnectCommands<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DisconnectCommands")
            .field("entity", &self.head)
            .finish_non_exhaustive()
    }
}

pub(crate) fn process_disconnections(
    mut disconnections: Query<(&mut PendingDisconnections, &FirewheelNode)>,
    targets: Query<&FirewheelNode>,
    node_map: Res<NodeMap>,
    mut context: ResMut<AudioContext>,
) {
    context.with(|context| {
        for (mut pending, source_node) in disconnections.iter_mut() {
            pending.0.retain(|disconnections| {
                let ports = disconnections.ports.as_deref().unwrap_or(DEFAULT_CONNECTION);

                let target_entity = match disconnections.target {
                    ConnectTarget::Entity(entity) => entity,
                    ConnectTarget::Label(label) => {
                        let Some(entity) = node_map.get(&label) else {
                            #[cfg(debug_assertions)]
                            {
                                let location = disconnections.origin;
                                error_once!("failed to disconnect from node label `{label:?}` at {location}: no associated Firewheel node found");
                            }
                            #[cfg(not(debug_assertions))]
                            error_once!("failed to disconnect from node label `{label:?}`: no associated Firewheel node found");

                            // We may need to wait for the intended label to be spawned.
                            return true;
                        };

                        *entity
                    }
                    ConnectTarget::Node(dest_node) => {
                        // no questions asked, simply disconnect
                        context.disconnect(source_node.0, dest_node, ports);

                        // if this fails, the target node must have been removed from the graph
                        return false;
                    }
                };

                let target = match targets.get(target_entity) {
                    Ok(t) => t,
                    Err(_) => {
                        #[cfg(debug_assertions)]
                        {
                            let location = disconnections.origin;
                            error_once!("failed to disconnect from entity `{target_entity:?}` at {location}: no Firewheel node found");
                        }
                        #[cfg(not(debug_assertions))]
                        error_once!("failed to disconnect from entity `{target_entity:?}`: no Firewheel node found");

                        return false;
                    }
                };

                context.disconnect(source_node.0, target.0, ports);

                false
            });
        }
    });
}

#[cfg(test)]
mod test {
    use crate::{
        connect::Connect, context::AudioContext, prelude::MainBus, profiling::ProfilingBackend,
        SeedlingPlugin,
    };

    use super::*;
    use bevy::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use firewheel::nodes::volume::VolumeNode;

    #[derive(Component)]
    struct One;
    #[derive(Component)]
    struct Two;

    fn prepare_app<F: IntoSystem<(), (), M>, M>(startup: F) -> App {
        let mut app = App::new();

        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            SeedlingPlugin::<ProfilingBackend> {
                default_pool_size: None,
                ..SeedlingPlugin::<ProfilingBackend>::new()
            },
        ))
        .add_systems(Startup, startup);

        app.finish();
        app.cleanup();
        app.update();

        app
    }

    fn run<F: IntoSystem<(), O, M>, O, M>(app: &mut App, system: F) -> O {
        let world = app.world_mut();
        world.run_system_once(system).unwrap()
    }

    #[test]
    fn test_disconnect() {
        let mut app = prepare_app(|mut commands: Commands| {
            commands
                .spawn((VolumeNode::default(), One))
                .chain_node((VolumeNode::default(), Two))
                .connect(MainBus);
        });

        // first, verify they're all connected
        run(
            &mut app,
            |mut context: ResMut<AudioContext>,
             one: Single<&FirewheelNode, With<One>>,
             two: Single<&FirewheelNode, With<Two>>,
             main: Single<&FirewheelNode, With<MainBus>>| {
                let one = one.into_inner();
                let two = two.into_inner();
                let main = main.into_inner();

                context.with(|context| {
                    // input node, output node, One, Two, and MainBus
                    assert_eq!(context.nodes().len(), 5);

                    let outgoing_edges_one: Vec<_> = context
                        .edges()
                        .into_iter()
                        .filter(|e| e.src_node == one.0)
                        .collect();
                    let outgoing_edges_two: Vec<_> = context
                        .edges()
                        .into_iter()
                        .filter(|e| e.src_node == two.0)
                        .collect();

                    assert_eq!(outgoing_edges_one.len(), 2);
                    assert_eq!(outgoing_edges_two.len(), 2);

                    assert!(outgoing_edges_one.iter().all(|e| e.dst_node == two.0));
                    assert!(outgoing_edges_two.iter().all(|e| e.dst_node == main.0));
                });
            },
        );

        // Then, apply a disconnection
        run(
            &mut app,
            |one: Single<Entity, With<One>>,
             two: Single<Entity, With<Two>>,
             mut commands: Commands| {
                let one = one.into_inner();
                let two = two.into_inner();

                commands.entity(one).disconnect(two);
            },
        );

        app.update();

        // finally, verify one and two are disconnected
        run(
            &mut app,
            |mut context: ResMut<AudioContext>,
             one: Single<&FirewheelNode, With<One>>,
             two: Single<&FirewheelNode, With<Two>>,
             main: Single<&FirewheelNode, With<MainBus>>| {
                let one = one.into_inner();
                let two = two.into_inner();
                let main = main.into_inner();

                context.with(|context| {
                    // input node, output node, One, Two, and MainBus
                    assert_eq!(context.nodes().len(), 5);

                    let outgoing_edges_one: Vec<_> = context
                        .edges()
                        .into_iter()
                        .filter(|e| e.src_node == one.0)
                        .collect();
                    let outgoing_edges_two: Vec<_> = context
                        .edges()
                        .into_iter()
                        .filter(|e| e.src_node == two.0)
                        .collect();

                    assert_eq!(outgoing_edges_one.len(), 0);
                    assert_eq!(outgoing_edges_two.len(), 2);

                    assert!(outgoing_edges_one.iter().all(|e| e.dst_node == two.0));
                    assert!(outgoing_edges_two.iter().all(|e| e.dst_node == main.0));
                });
            },
        );
    }
}
