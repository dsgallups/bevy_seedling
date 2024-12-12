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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MainBus;

impl NodeLabel for MainBus {
    fn dyn_clone(&self) -> Box<dyn NodeLabel> {
        Box::new(::core::clone::Clone::clone(self))
    }

    fn as_dyn_eq(&self) -> &dyn bevy_ecs::schedule::DynEq {
        self
    }

    fn dyn_hash(&self, mut state: &mut dyn ::core::hash::Hasher) {
        let ty_id = ::core::any::TypeId::of::<Self>();
        ::core::hash::Hash::hash(&ty_id, &mut state);
        ::core::hash::Hash::hash(self, &mut state);
    }
}

pub type InternedNodeLabel = Interned<dyn NodeLabel>;

#[derive(Component)]
pub struct InternedLabel(InternedNodeLabel);

impl InternedLabel {
    #[inline(always)]
    pub fn new(label: impl NodeLabel) -> Self {
        Self(label.intern())
    }
}
