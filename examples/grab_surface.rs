use bevy::color::palettes::css::{DARK_GREEN, GRAY, TAN, YELLOW};
use bevy::post_process::auto_exposure::AutoExposurePlugin;
use bevy::prelude::*;

use bevy_map_camera::{
    CameraControllerSettings, GrabSurface, LookTransform, MapCamera, MapCameraPlugin,
};

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, AutoExposurePlugin, MapCameraPlugin));

    app.add_systems(Startup, setup);
    app.add_systems(Update, draw_target);
    app.run();
}

fn setup(
    settings: Res<CameraControllerSettings>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ground, acts as a grab surface
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20., 20.))),
        MeshMaterial3d(materials.add(Color::from(DARK_GREEN))),
        GrabSurface,
    ));

    // raised plateau, acts as a grab surface
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(6.0, 2.0, 6.0))),
        MeshMaterial3d(materials.add(Color::from(TAN))),
        Transform::from_xyz(4.0, 1.0, -4.0),
        GrabSurface,
    ));

    // ramp up to the plateau, acts as a grab surface
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 0.2, 6.0))),
        MeshMaterial3d(materials.add(Color::from(TAN))),
        Transform::from_xyz(4.0, 1.0, 1.8).with_rotation(Quat::from_rotation_x(0.35)),
        GrabSurface,
    ));

    // a pillar that is NOT a grab surface, the camera pans through it
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.5, 4.0))),
        MeshMaterial3d(materials.add(Color::from(GRAY))),
        Transform::from_xyz(-4.0, 2.0, 3.0),
    ));

    // light
    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            range: 50.0,
            ..Default::default()
        },
        Transform::from_xyz(4.0, 12.0, 4.0),
    ));

    // camera
    commands.spawn((
        MapCamera,
        LookTransform::new(Vec3::new(1.0, 10.0, 14.0), Vec3::ZERO, Vec3::Y),
    ));

    // text
    commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        children![
            (
                Text(format!("Press {:?} to pan", settings.buttons.pan)),
                Node {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..Default::default()
                },
                Label,
            ),
            (
                Text(format!("Press {:?} to rotate", settings.buttons.rotate)),
                Node {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                Label,
            ),
            (
                Text::new("The ground, plateau and ramp are grab surfaces"),
                Node {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                Label,
            ),
        ],
    ));
}

/// Draws a marker at the camera target, which follows the grab surfaces
fn draw_target(mut gizmos: Gizmos, camera: Single<&LookTransform, With<MapCamera>>) {
    gizmos.sphere(
        Isometry3d::from_translation(camera.into_inner().target),
        0.15,
        YELLOW,
    );
}
