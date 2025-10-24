use std::time::Duration;

use bevy::color::palettes::css::{DARK_GREEN, TAN};
use bevy::post_process::auto_exposure::AutoExposurePlugin;
use bevy::prelude::*;

use bevy_easings::{CustomComponentEase, EaseMethod};
use bevy_map_camera::{LookTransform, MapCamera, MapCameraPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, AutoExposurePlugin, MapCameraPlugin));

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

    let cube_material = materials.add(Color::from(TAN));

    // cubes
    for x in -2..=2 {
        for z in -2..=2 {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::from_size(Vec3::splat(0.2)))),
                MeshMaterial3d(cube_material.clone()),
                Transform::from_xyz((x * 2) as f32, 0.1, (z * 2) as f32),
            ));
        }
    }

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // easing and camera
    let look_transform_from = LookTransform::new(
        Vec3 {
            x: 0.,
            y: 8.5,
            z: 10.0,
        },
        Vec3::ZERO,
        Vec3::Y,
    );

    let target = LookTransform::new(
        Vec3 {
            x: 10.,
            y: 8.5,
            z: 0.0,
        },
        Vec3::ZERO,
        Vec3::Y,
    );

    let easing = look_transform_from.ease_to(
        target,
        EaseMethod::Linear,
        bevy_easings::EasingType::PingPong {
            duration: Duration::from_secs(5),
            pause: None,
        },
    );

    commands.spawn((MapCamera, easing));
}
