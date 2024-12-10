use std::time::Duration;

use bevy::color::palettes::css::{DARK_GREEN, TAN};
use bevy::core_pipeline::auto_exposure::AutoExposurePlugin;
use bevy::prelude::*;

use bevy_map_camera::look_transform::LookTransformLens;
use bevy_map_camera::{LookTransform, MapCamera, MapCameraPlugin};
use bevy_tweening::{Animator, EaseMethod, RepeatCount, RepeatStrategy, Tween};

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

    let tween = Tween::new(
        EaseMethod::EaseFunction(EaseFunction::Linear),
        Duration::from_secs(5),
        LookTransformLens {
            start: look_transform_from,
            end: target,
        },
    )
    .with_repeat_strategy(RepeatStrategy::MirroredRepeat)
    .with_repeat_count(RepeatCount::Infinite);

    commands.spawn((MapCamera, Animator::new(tween)));
}
