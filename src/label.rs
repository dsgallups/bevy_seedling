//! Type-base node labelling.
//!
//! `bevy_seedling` provides a single label, [MainBus],
//! which represents the terminal node that every other
//! node must eventually reach.
//!
//! Any node that doesn't provide an explicit connection when spawned
//! will be automatically connected to [MainBus].
use crate::volume::Volume;
use bevy_ecs::{intern::Interned, prelude::*};

use crate::{AudioContext, ConnectNode};

bevy_ecs::define_label!(
    /// A label for addressing audio nodes.
    ///
    /// Types that implement [NodeLabel] can be used in place of entity IDs
    /// for audio node connections.
    /// ```
    /// # use crate::NodeLabel;
    /// #[derive(NodeLabel, Debug, Clone, PartialEq, Eq, Hash)]
    /// struct EffectsChain;
    ///
    /// fn system(server: Res<AssetServer>, mut commands: Commands) {
    ///     commands.spawn((Volume::new(0.25), InternedLabel::new(EffectsChain)));
    ///
    ///     commands
    ///         .spawn(SamplePlayer::new(server.load("sound.wav")))
    ///         .connect(EffectsChain);
    /// }
    /// ```
    NodeLabel,
    NODE_LABEL_INTERNER
);

/// The main audio bus.
///
/// All audio nodes must pass through this bus to
/// reach the output.
///
/// If no connections are specified for an entity
/// with a [Node][crate::Node] component, the
/// node will automatically be routed to this bus.
///
/// [MainBus] is a stereo volume node. To adjust the
/// global volume, you can query for a volume node's parameters
/// filtered on this label.
/// ```
/// fn mute(mut q: Single<&mut Params<VolumeParams>, With<MainBus>>) {
///     let mut params = q.into_inner();
///     params.gain.set(0.);
/// }
/// ```
#[derive(crate::NodeLabel, Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MainBus;

pub(crate) type InternedNodeLabel = Interned<dyn NodeLabel>;

/// A type-erased node label.
///
/// To associate a label with an audio node,
/// the node entity should be spawned with the label.
/// ```
/// # use crate::NodeLabel;
/// #[derive(NodeLabel, Debug, Clone, PartialEq, Eq, Hash)]
/// struct MyLabel;
/// # fn system(mut commands: Commands) {
///
/// commands.spawn((Volume::new(0.25), InternedLabel::new(MyLabel)));
/// # }
/// ```
#[derive(Component)]
pub struct InternedLabel(pub(crate) InternedNodeLabel);

impl InternedLabel {
    #[inline(always)]
    pub fn new(label: impl NodeLabel) -> Self {
        Self(label.intern())
    }
}

pub(crate) fn insert_main_bus(mut commands: Commands, mut context: ResMut<AudioContext>) {
    let terminal_node = context.with(|context| context.graph().graph_out_node());

    commands
        .spawn((Volume::new(1.), InternedLabel::new(MainBus), MainBus))
        .connect(terminal_node);
}
