# `bevy_map_camera`

A 3D camera controller inspired by Google Maps, [f4maps](https://demo.f4map.com/) and [Change Finder](https://chargefinder.com/nearby).

Based upon LookTransform, LookAngles and Orbital Camera Controller from [`smooth-bevy-cameras`](https://github.com/bonsairobo/smooth-bevy-cameras).

## Features

- Orbital camera
- Zoom towards pointer
- Grab pan
- Focus point follows XZ-plane
- Support for Perspective and Orthographic projection
- Smoothed movement
- Customizable keyboard/mouse controls
- Touch support

## Usage

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
