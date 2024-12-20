A sprouting integration of the [Firewheel](https://github.com/BillyDM/firewheel)
audio engine for [Bevy](https://bevyengine.org/).

**NOTE**: this crate is very much a work-in-progress.
The APIs are not set in stone, and not all platforms
are supported (notably `wasm`).

## Getting started

First, you'll need to add the dependency to your `Cargo.toml`.

```toml
[dependencies]
bevy_seedling = "0.1"

# At this stage, it may be better to track the main branch
[dependencies]
bevy_seedling = { git = "https://github.com/corvusprudens/bevy_seedling" }
```

Then, you'll need to add the [`SeedlingPlugin`] to your app.

```no_test
use bevy::prelude::*;
use bevy_seedling::SeedlingPlugin;

fn main() {
    App::default()
        .add_plugins((DefaultPlugins, SeedlingPlugin::default()))
        .run();
}
```

[The repository's examples](https://github.com/CorvusPrudens/bevy_seedling/tree/master/examples)
should help you get up to speed on common usage patterns.

## Overview

`bevy_seedling` provides a thin ECS wrapper over `Firewheel`.

A `Firewheel` audio node is typically represented in the ECS as
an entity with a [`Node`] and a component that can generate
`Firewheel` events such as [`VolumeNode`].

Interactions with the audio engine are buffered.
This includes inserting nodes into the audio graph,
removing nodes from the graph, making connections 
between nodes, and sending node events. This provides
a few advantages:

1. Audio entities do not need to wait until
   they have Firewheel IDs before they can 
   make connections.
2. Systems that spawn or interact with
   audio entities can be trivially parallelized.
3. Graph-mutating interactions are properly deferred
   while the audio graph isn't ready, for example
   if it's been temporarily deactiviated.

The main disadvantage is potentially increased latency,
since several milliseconds may pass between an
audio-event-generating system and actually passing
those events to the audio engine. However, since
`bevy_seedling` provides direct access to the audio context,
you can always immediately queue and flush events
if necessary.