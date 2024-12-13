use arrayvec::ArrayVec;
use bevy_math::prelude::EasingCurve;
use core::any::Any;
use firewheel::clock::ClockSeconds;
use smallvec::SmallVec;

pub enum ContinuousEvent<T> {
    Immediate(T),
    Deferred(T, ClockSeconds),
    Curve {
        curve: EasingCurve<T>,
        start: ClockSeconds,
        end: ClockSeconds,
    },
}

pub struct Continuous<T> {
    value: T,
    events: ArrayVec<ContinuousEvent<T>, 4>,
}

pub enum DeferredEvent<T> {
    Immediate(T),
    Deferred(T, ClockSeconds),
}

pub struct Deferred<T> {
    value: T,
    events: ArrayVec<DeferredEvent<T>, 4>,
}

pub enum MessageData {
    F32(ContinuousEvent<f32>),
    F64(ContinuousEvent<f64>),
    I32(ContinuousEvent<i32>),
    I64(ContinuousEvent<i64>),
    Bool(DeferredEvent<bool>),
    Any(Box<dyn Any + Send>),
}

pub struct Message {
    data: MessageData,
    path: ParamPath,
}

pub struct Messages(Vec<Message>);

impl Messages {
    pub fn push(&mut self, message: Message) {
        self.0.push(message);
    }
}

#[derive(Clone)]
pub struct ParamPath(SmallVec<[u16; 8]>);

impl ParamPath {
    fn with(&self, index: u16) -> Self {
        let mut new = self.0.clone();
        new.push(index);
        Self(new)
    }
}

pub enum PatchError {
    InvalidPath,
    InvalidData,
}

pub trait AudioParam: Sized {
    fn to_messages(&self, cmp: &Self, messages: &mut Messages, path: ParamPath);

    fn patch(&mut self, data: MessageData, path: &[u16]) -> Result<(), PatchError>;

    fn tick(&mut self, time: ClockSeconds);
}

impl AudioParam for f32 {
    fn to_messages(&self, cmp: &Self, messages: &mut Messages, path: ParamPath) {
        if self != cmp {
            messages.push(Message {
                data: MessageData::F32(ContinuousEvent::Immediate(*self)),
                path: path.clone(),
            });
        }
    }

    fn patch(&mut self, data: MessageData, path: &[u16]) -> Result<(), PatchError> {
        match data {
            MessageData::F32(ContinuousEvent::Immediate(value)) => {
                *self = value;

                Ok(())
            }
            _ => Err(PatchError::InvalidData),
        }
    }

    fn tick(&mut self, _time: ClockSeconds) {}
}

impl AudioParam for bool {
    fn to_messages(&self, cmp: &Self, messages: &mut Messages, path: ParamPath) {
        if self != cmp {
            messages.push(Message {
                data: MessageData::Bool(DeferredEvent::Immediate(*self)),
                path: path.clone(),
            });
        }
    }

    fn patch(&mut self, data: MessageData, path: &[u16]) -> Result<(), PatchError> {
        match data {
            MessageData::Bool(DeferredEvent::Immediate(value)) => {
                *self = value;

                Ok(())
            }
            _ => Err(PatchError::InvalidData),
        }
    }

    fn tick(&mut self, _time: ClockSeconds) {}
}

impl AudioParam for Continuous<f32> {
    fn to_messages(&self, cmp: &Self, messages: &mut Messages, path: ParamPath) {
        todo!()
    }

    fn patch(&mut self, data: MessageData, path: &[u16]) -> Result<(), PatchError> {
        match data {
            MessageData::F32(message) => {
                self.events.push(message);

                Ok(())
            }
            _ => Err(PatchError::InvalidData),
        }
    }

    fn tick(&mut self, time: ClockSeconds) {
        todo!()
    }
}

#[derive(crate::AudioParam)]
struct ExampleParams {
    volume: Continuous<f32>,
    frequency: Continuous<f32>,
    freeze: bool,
}

// impl AudioParam for ExampleParams {
//     fn to_messages(&self, cmp: &Self, messages: &mut Messages, path: ParamPath) {
//         self.volume.to_messages(&cmp.volume, messages, path.with(0));
//         self.frequency
//             .to_messages(&cmp.frequency, messages, path.with(1));
//         self.freeze.to_messages(&cmp.freeze, messages, path.with(2));
//     }

//     fn patch(&mut self, data: MessageData, path: &[u16]) -> Result<(), PatchError> {
//         match path.first() {
//             Some(0) => self.volume.patch(data, &path[1..]),
//             Some(1) => self.volume.patch(data, &path[1..]),
//             Some(2) => self.volume.patch(data, &path[1..]),
//             _ => Err(PatchError::InvalidPath),
//         }
//     }

//     fn tick(&mut self, time: ClockSeconds) {
//         self.volume.tick(time);
//         self.frequency.tick(time);
//         self.freeze.tick(time);
//     }
// }
