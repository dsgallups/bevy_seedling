#![allow(clippy::type_complexity)]

use bevy_app::{Last, Plugin};
use bevy_asset::{AssetApp, Assets};
use bevy_ecs::prelude::*;
use bevy_log::{error, info};
use firewheel::{
    clock::EventDelay, node::NodeEvent, sampler::one_shot::OneShotSamplerNode, FirewheelConfig,
    FirewheelCpalCtx, UpdateStatus,
};
use std::sync::mpsc;

pub mod sample;

#[derive(Default)]
pub struct SeedlingPlugin {
    pub settings: FirewheelConfig,
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct LoadingSample;

fn on_add(
    q: Query<
        (Entity, &sample::SamplePlayer),
        (Added<sample::SamplePlayer>, Without<LoadingSample>),
    >,
    context: Res<AudioContext>,
    mut commands: Commands,
    assets: Res<Assets<sample::Sample>>,
) {
    context.with(|context| {
        for (entity, player) in q.iter() {
            if let Some(asset) = assets.get(&player.0) {
                if let Some(graph) = context.graph_mut() {
                    let sampler_node = graph
                        .add_node(OneShotSamplerNode::new(Default::default()).into(), None)
                        .unwrap();

                    graph
                        .connect(
                            sampler_node,
                            graph.graph_out_node(),
                            &[(0, 0), (1, 1)],
                            false,
                        )
                        .unwrap();

                    graph.queue_event(NodeEvent {
                        node_id: sampler_node,
                        delay: EventDelay::Immediate,
                        event: firewheel::node::NodeEventType::PlaySample {
                            sample: asset.get(),
                            normalized_volume: 1.0,
                            stop_other_voices: false,
                        },
                    });
                }
            } else {
                commands.entity(entity).insert(LoadingSample);
            }
        }
    });
}

fn loading(
    q: Query<(Entity, &sample::SamplePlayer), With<LoadingSample>>,
    context: Res<AudioContext>,
    mut commands: Commands,
    assets: Res<Assets<sample::Sample>>,
) {
    context.with(|context| {
        for (entity, player) in q.iter() {
            if let Some(asset) = assets.get(&player.0) {
                if let Some(graph) = context.graph_mut() {
                    info!("Playing sound");
                    let sampler_node = graph
                        .add_node(OneShotSamplerNode::new(Default::default()).into(), None)
                        .unwrap();

                    graph
                        .connect(
                            sampler_node,
                            graph.graph_out_node(),
                            &[(0, 0), (1, 1)],
                            false,
                        )
                        .unwrap();

                    graph.queue_event(NodeEvent {
                        node_id: sampler_node,
                        delay: EventDelay::Immediate,
                        event: firewheel::node::NodeEventType::PlaySample {
                            sample: asset.get(),
                            normalized_volume: 1.0,
                            stop_other_voices: false,
                        },
                    });
                }

                commands.entity(entity).remove::<LoadingSample>();
            }
        }
    });
}

fn update_graph(context: Res<AudioContext>) {
    context.with(|context| {
        match context.update() {
            UpdateStatus::Inactive => {}
            UpdateStatus::Active { graph_error } => {
                if let Some(e) = graph_error {
                    error!("graph error: {}", e);
                }
            }
            UpdateStatus::Deactivated { error, .. } => {
                error!("Deactivated unexpectedly: {:?}", error);
            }
        }
        context.flush_events();
    });
}

type ThreadLocalCall = Box<dyn FnOnce(&mut FirewheelCpalCtx) + Send + 'static>;

#[derive(Debug, Resource)]
pub struct AudioContext(mpsc::Sender<ThreadLocalCall>);

impl AudioContext {
    pub fn with<F, O>(&self, f: F) -> O
    where
        F: FnOnce(&mut FirewheelCpalCtx) -> O + Send,
        O: Send + 'static,
    {
        let (send, receive) = mpsc::sync_channel(1);
        let func: Box<dyn FnOnce(&mut FirewheelCpalCtx) + Send> = Box::new(move |ctx| {
            let result = f(ctx);
            send.send(result).unwrap();
        });

        // # SAFETY
        //
        // This thread will block until the function returns,
        // so we can pretend it has a static lifetime.
        let func = unsafe {
            core::mem::transmute::<
                Box<dyn FnOnce(&mut FirewheelCpalCtx) + Send>,
                Box<dyn FnOnce(&mut FirewheelCpalCtx) + Send + 'static>,
            >(func)
        };

        self.0.send(func).unwrap();
        receive.recv().unwrap()
    }
}

impl Plugin for SeedlingPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        let (bev_to_audio_tx, bev_to_audio_rx) = mpsc::channel::<ThreadLocalCall>();
        let settings = self.settings;
        std::thread::spawn(move || {
            let mut context = FirewheelCpalCtx::new(settings);
            context
                .activate(Default::default())
                .expect("failed to activate the audio context");

            while let Ok(func) = bev_to_audio_rx.recv() {
                (func)(&mut context);
            }
        });

        let context = AudioContext(bev_to_audio_tx);
        let sample_rate = context.with(|ctx| ctx.stream_info().unwrap().sample_rate);

        app.insert_resource(context)
            .register_asset_loader(sample::SampleLoader { sample_rate })
            .init_asset::<sample::Sample>()
            .add_systems(Last, (on_add, loading, update_graph).chain());
    }
}
