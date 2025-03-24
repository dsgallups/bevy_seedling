//! All of `bevy_seedling`'s audio nodes.

use crate::{prelude::RegisterNode, SeedlingSystems};
use bevy_ecs::prelude::*;

pub mod bpf;
pub mod freeverb;
pub mod lpf;
pub mod send;

/// Registration and logic for `bevy_seedling`'s audio nodes.
pub(crate) struct SeedlingNodesPlugin;

impl bevy_app::Plugin for SeedlingNodesPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.register_node::<bpf::BandPassNode>()
            .register_node::<lpf::LowPassNode>()
            .register_node::<send::SendNode>()
            .register_node::<freeverb::FreeverbNode>()
            .add_systems(
                bevy_app::Last,
                (send::connect_sends, send::update_remote_sends).before(SeedlingSystems::Acquire),
            );
    }
}
