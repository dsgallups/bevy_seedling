#![allow(clippy::type_complexity)]

use bevy_app::{Last, Plugin};
use bevy_asset::AssetApp;
use bevy_ecs::{intern::Interned, prelude::*};
use firewheel::FirewheelConfig;

pub mod context;
pub mod node;
pub mod sample;

pub use context::AudioContext;
pub use node::{ConnectNode, ConnectTarget, Node};

/// Sets for all `bevy_seedling` systems.
///
/// These are all inserted into the [Last] schedule.
#[derive(Debug, SystemSet, PartialEq, Eq, Hash, Clone)]
pub enum SeedlingSystems {
    /// Entities without audio nodes acquire them from the audio context.
    Acquire,
    /// Pending connections are made.
    Connect,
    /// Queue audio engine events.
    ///
    /// While it's not strictly necessary to separate this
    /// set from [SeedlingSystems::Connect], it's a nice
    /// semantics separation.
    Queue,
    /// The audio context is updated and flushed.
    Flush,
}

bevy_ecs::define_label!(
    /// A label for addressing Firewheel audio nodes.
    NodeLabel,
    NODE_LABEL_INTERNER
);

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

#[derive(Default)]
pub struct SeedlingPlugin {
    pub settings: FirewheelConfig,
}

impl Plugin for SeedlingPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        let mut context = AudioContext::new(self.settings);
        let (sample_rate, out_id) = context.with(|ctx| {
            (
                ctx.stream_info().unwrap().sample_rate,
                ctx.graph().graph_out_node(),
            )
        });

        let node_map = node::NodeMap::new(out_id);

        app.insert_resource(context)
            .configure_sets(
                Last,
                (
                    SeedlingSystems::Connect.after(SeedlingSystems::Acquire),
                    SeedlingSystems::Queue.after(SeedlingSystems::Acquire),
                    SeedlingSystems::Flush
                        .after(SeedlingSystems::Connect)
                        .after(SeedlingSystems::Queue),
                ),
            )
            .insert_resource(node_map)
            .register_asset_loader(sample::SampleLoader { sample_rate })
            .init_asset::<sample::Sample>()
            .add_systems(
                Last,
                (
                    sample::on_add_sample.in_set(SeedlingSystems::Acquire),
                    node::process_connections.in_set(SeedlingSystems::Connect),
                    sample::trigger_pending_samples.in_set(SeedlingSystems::Queue),
                    context::update_context.in_set(SeedlingSystems::Flush),
                ),
            );
    }
}
