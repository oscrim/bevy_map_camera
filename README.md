# `bevy_map_camera`

[![docs.rs](https://docs.rs/bevy_map_camera/badge.svg)](https://docs.rs/bevy_map_camera)
[![crates.io](https://img.shields.io/crates/v/bevy_map_camera)](https://crates.io/crates/bevy_map_camera)

A 3D camera controller inspired by Google Maps, [f4maps](https://demo.f4map.com/) and [Charge Finder](https://chargefinder.com/nearby).

![bevy_map_camera example](https://github.com/user-attachments/assets/1ac13767-9ad9-495f-90fd-9f8b765347ba)

Based upon LookTransform, LookAngles and Orbital Camera Controller from [`smooth-bevy-cameras`](https://github.com/bonsairobo/smooth-bevy-cameras).

## Features

- Orbital camera
- Zoom towards pointer
- Grab pan
- Camera target follows XZ-plane
- Support for Perspective and Orthographic projection
- Smoothed movement
- Customizable keyboard/mouse controls
- Touch support
  - One finger pan
  - Two finger rotate
  - Pinch to zoom
- Supports Easing though [`bevy_easings`](https://github.com/vleue/bevy_easings), part of `default` features.
- Supports Tweening through [`bevy_tweening`](https://github.com/djeedai/bevy_tweening), requires `bevy_tweening` feature.

## Quick Start

```rs
use bevy::prelude::*;

use bevy_map_cam::{CameraBundle, LookTransform, MapCameraPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(MapCameraPlugin::default());

    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
) {
        commands.spawn(CameraBundle::new_with_transform(LookTransform::new(
        Vec3 {
            x: 1.,
            y: 2.5,
            z: 5.0,
        },
        Vec3::ZERO,
        Vec3::Y,
    )));
}
```

Check out the [projection example](https://github.com/oscrim/bevy_map_camera/blob/main/examples/projection.rs) to see how to change between Perspective and Orthographic.

## Compatible Bevy versions

The `main` branch is compatible with the latest Bevy release.

| `bevy_map_camera` | `bevy` |
| :--               | :--    |
| `0.1`             | `0.14` |
