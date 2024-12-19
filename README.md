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

Most interactions with the audio engine are buffered.
