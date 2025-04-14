use bevy_animation::animated_field;
use bevy_math::Curve;

pub struct Tween<T> {
    curve: Box<dyn Curve<T>>,
}

fn test() {
    use crate::prelude::*;
    use bevy_animation::prelude::*;

    let field = animated_field!(VolumeNode::volume);
}
