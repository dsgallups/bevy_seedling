use bevy_ecs::{intern::Interned, prelude::*};

bevy_ecs::define_label!(
    /// A label for addressing Firewheel audio nodes.
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
#[derive(crate::NodeLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MainBus;

pub type InternedNodeLabel = Interned<dyn NodeLabel>;

#[derive(Component)]
pub struct InternedLabel(InternedNodeLabel);

impl InternedLabel {
    #[inline(always)]
    pub fn new(label: impl NodeLabel) -> Self {
        Self(label.intern())
    }
}
