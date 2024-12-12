use bevy_ecs::prelude::*;
use bevy_log::error;
use firewheel::{FirewheelConfig, FirewheelCpalCtx, UpdateStatus};
use std::sync::mpsc;

#[derive(Debug, Resource)]
pub struct AudioContext(mpsc::Sender<ThreadLocalCall>);

impl AudioContext {
    /// Spawn the audio process and control thread.
    pub fn new(settings: FirewheelConfig) -> Self {
        let (bev_to_audio_tx, bev_to_audio_rx) = mpsc::channel::<ThreadLocalCall>();
        std::thread::spawn(move || {
            let mut context = FirewheelCpalCtx::new(settings);
            context
                .activate(Default::default())
                .expect("failed to activate the audio context");

            while let Ok(func) = bev_to_audio_rx.recv() {
                (func)(&mut context);
            }
        });

        AudioContext(bev_to_audio_tx)
    }
}

pub type ThreadLocalCall = Box<dyn FnOnce(&mut FirewheelCpalCtx) + Send + 'static>;

impl AudioContext {
    /// Send `f` to the underlying control thread to operate on the audio context.
    ///
    /// This call will block until `f` returns.
    ///
    /// This method takes a mutable reference to `self` to prevent trivial deadlocks.
    /// This API can't completely prevent them in the general case: calling
    /// [AudioContext::with] within itself will deadlock.
    pub fn with<F, O>(&mut self, f: F) -> O
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

pub(crate) fn update_context(mut context: ResMut<AudioContext>) {
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
