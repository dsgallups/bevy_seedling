//! This example demonstrates how to play a one-shot sample.

use bevy::{log::LogPlugin, prelude::*};
use bevy_seedling::{sample::SamplePlayer, SeedlingPlugin};
use firewheel::{
    diff::{Diff, EventQueue, Patch, PatchError, PathBuilder},
    event::{NodeEventType, ParamData},
};

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            LogPlugin::default(),
            AssetPlugin::default(),
            SeedlingPlugin::default(),
        ))
        .add_systems(
            Startup,
            |server: Res<AssetServer>, mut commands: Commands| {
                // Spawning a `SamplePlayer` component will play a sample
                // once as soon as it's loaded.
                //
                // The sample will be assigned to one of the playback nodes
                // connected to the `SamplePoolBus`.
                commands.spawn(SamplePlayer::new(server.load("snd_wobbler.wav")));
            },
        )
        .run();
}

enum Manual {
    Unit,
    Tuple(f32, f32),
    Struct { a: f32, b: f32 },
}

impl Diff for Manual {
    fn diff<E: EventQueue>(&self, baseline: &Self, path: PathBuilder, event_queue: &mut E) {
        match (self, baseline) {
            (Manual::Unit, Manual::Unit) => {}
            (Manual::Unit, _) => {
                let path = path.with(0);

                event_queue.push(NodeEventType::Param {
                    data: ParamData::U32(0),
                    path: path.build(),
                });
            }
            (Manual::Tuple(a0, b0), Manual::Tuple(a1, b1)) => {
                let path = path.with(1);

                a0.diff(a1, path.with(0), event_queue);
                b0.diff(b1, path.with(1), event_queue);
            }
            (Manual::Tuple(a0, b0), _) => {
                let path = path.with(1);

                event_queue.push(NodeEventType::Param {
                    data: ParamData::any((*a0, *b0)),
                    path: path.build(),
                });
            }
            (Manual::Struct { a: a0, b: b0 }, Manual::Struct { a: a1, b: b1 }) => {
                let path = path.with(2);

                a0.diff(a1, path.with(0), event_queue);
                b0.diff(b1, path.with(1), event_queue);
            }
            (Manual::Struct { a, b }, _) => {
                let path = path.with(2);

                event_queue.push(NodeEventType::Param {
                    data: ParamData::any((*a, *b)),
                    path: path.build(),
                });
            }
        }
    }
}

impl Patch for Manual {
    fn patch(&mut self, data: &ParamData, path: &[u32]) -> Result<(), PatchError> {
        match (path, self) {
            ([0], s) => {
                // because this variant has no data, there's no need to even look at it
                *s = Manual::Unit;

                Ok(())
            }
            ([1], s) => {
                // with a terminal path, we must be setting both parameters
                let (a, b): &(f32, f32) = data.downcast_ref().ok_or(PatchError::InvalidData)?;

                *s = Manual::Tuple(*a, *b);
                Ok(())
            }
            ([1, 0, tail @ ..], Self::Tuple(a, _)) => a.patch(data, tail),
            ([1, 1, tail @ ..], Self::Tuple(_, b)) => b.patch(data, tail),
            ([2], s) => {
                let (a, b): &(f32, f32) = data.downcast_ref().ok_or(PatchError::InvalidData)?;

                *s = Manual::Struct { a: *a, b: *b };
                Ok(())
            }
            ([2, 0, tail @ ..], Self::Struct { a, .. }) => a.patch(data, tail),
            ([2, 1, tail @ ..], Self::Struct { b, .. }) => b.patch(data, tail),
            _ => return Err(PatchError::InvalidPath),
        }
    }
}

#[derive(Diff)]
enum Automatic<T> {
    Unit,
    Tuple(f32, T),
    Struct { a: f32, b: NonClone },
}
