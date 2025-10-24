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
  - Configurable height
- Camera target follows XZ-plane
- Smoothed movement
- Customizable keyboard/mouse controls
- Touch support
  - One finger pan
  - Two finger rotate
  - Pinch to zoom
- Supports Easing though [`bevy_easings`](https://github.com/vleue/bevy_easings), requires `easings` feature.
  - Implemented for `LookTransform`
- Supports Tweening through [`bevy_tweening`](https://github.com/djeedai/bevy_tweening), requires `tweening` feature.
  - Lenses
    - `LookTransformLens`
    - `GrabHeightLens`

## Quick Start

```rs
use bevy::prelude::*;

use bevy_map_cam::{MapCamera, LookTransform, MapCameraPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(MapCameraPlugin::default());

    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10., 10.))),
        MeshMaterial3d(materials.add(Color::from(DARK_GREEN))),
    ));

    // Camera
    commands.spawn(MapCamera);
}
```

## Compatible Bevy versions

| bevy_map_camera | bevy |
| :--             | :--  |
| 0.4             | 0.17 |
| 0.3             | 0.16 |
| 0.2             | 0.15 |
| 0.1             | 0.14 |
