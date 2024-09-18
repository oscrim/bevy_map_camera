use bevy::color::palettes::css::{DARK_GREEN, TAN};
use bevy::core_pipeline::auto_exposure::AutoExposurePlugin;
use bevy::prelude::*;

use bevy_map_camera::{CameraControllerSettings, LookTransform, MapCameraBundle, MapCameraPlugin};

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
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(10., 10.)),
        material: materials.add(Color::from(DARK_GREEN)),
        ..Default::default()
    });

    let cube_material = materials.add(Color::from(TAN));

    // cubes
    for x in -2..=2 {
        for z in -2..=2 {
            commands.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::from_size(Vec3::splat(0.2))),
                material: cube_material.clone(),
                transform: Transform::from_xyz((x * 2) as f32, 0.1, (z * 2) as f32),
                ..Default::default()
            });
        }
    }

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    // camera
    commands.spawn(MapCameraBundle::new_with_transform(LookTransform::new(
        Vec3 {
            x: 1.,
            y: 8.5,
            z: 10.0,
        },
        Vec3::ZERO,
        Vec3::Y,
    )));

    // text
    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    format!("Press {:?} to pan", settings.buttons.pan),
                    TextStyle::default(),
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..Default::default()
                }),
                Label,
            ));

            parent.spawn((
                TextBundle::from_section(
                    format!("Press {:?} to rotate", settings.buttons.rotate),
                    TextStyle::default(),
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                }),
                Label,
            ));
        });
}
