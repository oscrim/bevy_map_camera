use bevy::color::palettes::css::{DARK_GREEN, TAN};
use bevy::post_process::auto_exposure::AutoExposurePlugin;
use bevy::prelude::*;

use bevy_map_camera::{CameraControllerSettings, LookTransform, MapCamera, MapCameraPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, AutoExposurePlugin, MapCameraPlugin));

    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    settings: Res<CameraControllerSettings>,
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

    // camera
    commands.spawn((
        MapCamera,
        LookTransform::new(
            Vec3 {
                x: 1.,
                y: 8.5,
                z: 10.0,
            },
            Vec3::ZERO,
            Vec3::Y,
        ),
    ));

    // text
    commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text(format!("Press {:?} to pan", settings.buttons.pan)),
                Node {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..Default::default()
                },
                Label,
            ));

            parent.spawn((
                Text(format!("Press {:?} to rotate", settings.buttons.rotate)),
                Node {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                Label,
            ));
        });
}
