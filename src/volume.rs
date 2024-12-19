//! Volume node.

use crate::node::{EcsNode, Events, Params};
use bevy_ecs::component::ComponentId;
use bevy_ecs::prelude::*;
use bevy_ecs::world::DeferredWorld;
use firewheel::basic_nodes::{VolumeNode, VolumeParams};
use firewheel::node::{AudioNode, Continuous};

/// A simple volume node.
#[derive(Component)]
#[component(on_insert = on_insert)]
#[require(Events)]
pub struct Volume(VolumeParams);

impl Volume {
    pub fn new(volume: f32) -> Self {
        Volume(VolumeParams {
            gain: Continuous::new(volume),
        })
    }
}

fn on_insert(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
    let params = world.get::<Volume>(entity).unwrap().0.clone();
    world.commands().entity(entity).insert(Params::new(params));
}

impl EcsNode for Volume {
    fn node(&self) -> Box<dyn AudioNode> {
        VolumeNode::new(self.0.clone()).into()
    }
}
