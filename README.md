[![crates.io](https://img.shields.io/crates/v/bevy_seedling)](https://crates.io/crates/bevy_seedling)
[![docs.rs](https://docs.rs/bevy_seedling/badge.svg)](https://docs.rs/bevy_seedling)

A sprouting integration of the [Firewheel](https://github.com/BillyDM/firewheel)
audio engine for [Bevy](https://bevyengine.org/).

## Getting started

First, you'll need to add the dependency to your `Cargo.toml`.
Note that you'll need to disable Bevy's "bevy_audio" feature,
meaning you'll need to specify quite a few features
manually!

```toml
[dependencies]
bevy_seedling = "0.3"
bevy = { version = "0.15", default-features = false, features = [
  "bevy_asset",
  "bevy_state",
  # ...
] }
```

[See here](https://docs.rs/crate/bevy/latest/features) for a list
of Bevy's default features.

Then, you'll need to add the `SeedlingPlugin` to your app.

```rs
use bevy::prelude::*;
use bevy_seedling::prelude::*;

fn main() {
    App::default()
        .add_plugins((DefaultPlugins, SeedlingPlugin::default()))
        .run();
}
```

[The repository's examples](https://github.com/CorvusPrudens/bevy_seedling/tree/master/examples)
should help you get up to speed on common usage patterns.

## Overview

Once you've registered the plugin, playing a sample is easy!

```rs
fn play(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn(SamplePlayer::new(server.load("my_sample.wav")));
}
```

`PlaybackSettings` gives you some
control over how your samples are played.

```rs
fn play_with_settings(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn((
        SamplePlayer::new(server.load("my_sample.wav")),
        PlaybackSettings::LOOP,
    ));
}
```

By default, sample players are queued up in the default sample pool,
`DefaultPool`. If you'd like to apply effects to your
samples, you can define a new pool with per-sampler effects.

```rs
fn custom_pool(mut commands: Commands, server: Res<AssetServer>) {
    // First, you'll need a label.
    #[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
    struct MyPool;

    // Let's spawn a pool with spatial audio and four samplers.
    Pool::new(MyPool, 4)
        .effect(SpatialBasicNode::default())
        .spawn(&mut commands);

    // To play a sample in this pool, just spawn a sample
    // player with its label.
    commands.spawn((
        MyPool,
        SamplePlayer::new(server.load("my_sample.wav")),
    ));
}
```

You can also define free-standing effects chains and
connect multiple pools to it.

```rs
fn chains(mut commands: Commands, server: Res<AssetServer>) {
    // We can also define labels for individual nodes.
    #[derive(NodeLabel, Debug, Clone, PartialEq, Eq, Hash)]
    struct UnderwaterEffects;

    commands.spawn((
        UnderwaterEffects,
        // We'll use a low-pass filter to simulate sounds underwater
        LowPassNode::new(1000.0),
    ))
    // Let's chain it into a volume node so everything's
    // a little quieter.
    .chain_node(VolumeNode {
        volume: Volume::Linear(0.5),
    });

    // Finally, we'll create a couple sample pools and connect
    // them to our water effects.
    #[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
    struct MusicPool;

    Pool::new(MusicPool, 1)
        .spawn(&mut commands)
        .connect(UnderwaterEffects);

    #[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash)]
    struct SfxPool;

    Pool::new(SfxPool, 16)
        .spawn(&mut commands)
        .connect(UnderwaterEffects);
}
```

## Custom nodes

`bevy_seedling` is designed to make authoring custom nodes breeze!
For an introduction, check out the [custom node example](https://github.com/CorvusPrudens/bevy_seedling/blob/master/examples/custom_node.rs)
in the repository.

## Design

`bevy_seedling` provides a thin ECS wrapper over `Firewheel`.

A `Firewheel` audio node is typically represented in the ECS as
an entity with a `FirewheelNode` and a component that can generate
`Firewheel` events, such as `VolumeNode`.

Interactions with the audio engine are buffered.
This includes inserting nodes into the audio graph,
removing nodes from the graph, making connections
between nodes, and sending node events. This provides
a few advantages:

1. Audio entities do not need to wait until
   they have Firewheel IDs before they can
   make connections or generate events.
2. Systems that spawn or interact with
   audio entities can be trivially parallelized.
3. Graph-mutating interactions are properly deferred
   while the audio graph isn't ready, for example
   if it's been temporarily deactivated.

## Bevy version compatibility

| `bevy` | `bevy_seedling` |
| ------ | --------------- |
| 0.15   | 0.3             |

## Future work

- Graph operations

  Audio entities currently support only a subset of the underlying
  Firewheel graph API. In particular, the ability to disconnect nodes will
  need to be added.

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
