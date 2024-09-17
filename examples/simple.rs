use bevy::prelude::*;

use bevy_map_cam::{CameraBundle, CameraControllerSettings, LookTransform, MapCameraPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(MapCameraPlugin::default());

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
        mesh: meshes.add(Plane3d::default().mesh().size(5., 5.)),
        material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
        ..Default::default()
    });

    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(1.0)))),
        material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    // camera
    commands.spawn(CameraBundle::new_with_transform(LookTransform::new(
        Vec3 {
            x: 1.,
            y: 2.5,
            z: 5.0,
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
                    TextStyle {
                        font_size: 30.0,
                        color: Color::BLACK,
                        ..Default::default()
                    },
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
                    TextStyle {
                        font_size: 30.0,
                        color: Color::BLACK,
                        ..Default::default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(5.0)),
                    align_self: AlignSelf::FlexStart,
                    ..default()
                }),
                Label,
            ));
        });
}
