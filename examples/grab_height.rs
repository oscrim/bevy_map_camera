use std::f32::consts::PI;
use std::time::Duration;

use bevy::color::palettes::css::{DARK_GREEN, TAN};
use bevy::core_pipeline::auto_exposure::AutoExposurePlugin;
use bevy::prelude::*;

use bevy_input::common_conditions::input_just_pressed;
use bevy_map_camera::controller::GrabHeightLens;
use bevy_map_camera::{
    CameraController, CameraControllerSettings, LookTransform, MapCamera, MapCameraPlugin,
};
use bevy_tweening::{Animator, EaseMethod, RepeatCount, RepeatStrategy, Tween};

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, AutoExposurePlugin, MapCameraPlugin));

    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            (update_grab_height, draw_plane).chain(),
            toggle_height_animation.run_if(input_just_pressed(KeyCode::Space)),
        ),
    );
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

            parent.spawn((
                Text::new("Press ArrowUp to increase height"),
                Node {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                Label,
            ));

            parent.spawn((
                Text::new("Press ArrowDown to decrease height"),
                Node {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                Label,
            ));

            parent.spawn((
                Text::new("Press Space to toggle height animation"),
                Node {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                Label,
            ));
        });
}

fn update_grab_height(
    mut controller_query: Query<&mut CameraController>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut controller = controller_query.single_mut();
    let height_change = 2.0 * time.delta().as_secs_f32();
    if keys.pressed(KeyCode::ArrowUp) {
        controller.grab_height += height_change;
    } else if keys.pressed(KeyCode::ArrowDown) {
        controller.grab_height -= height_change;
    }
}

fn toggle_height_animation(
    mut commands: Commands,
    controller_query: Query<(Entity, &CameraController, Has<Animator<CameraController>>)>,
) {
    let (entity, controller, animation_runing) = controller_query.single();

    if animation_runing {
        commands
            .entity(entity)
            .remove::<Animator<CameraController>>();
        return;
    }

    let tween = Tween::new(
        EaseMethod::EaseFunction(EaseFunction::Linear),
        Duration::from_secs(5),
        GrabHeightLens {
            start: controller.grab_height,
            end: controller.grab_height + 5.0,
        },
    )
    .with_repeat_strategy(RepeatStrategy::MirroredRepeat)
    .with_repeat_count(RepeatCount::Infinite);

    commands.entity(entity).insert(Animator::new(tween));
}

fn draw_plane(mut gizmos: Gizmos, controller_query: Query<&CameraController>) {
    let translation = Vec3::Y * controller_query.single().grab_height;

    let isometry = Isometry3d::new(translation, Quat::from_axis_angle(Vec3::X, PI / 2.0));

    gizmos.grid(
        isometry,
        UVec2::new(10, 10),
        Vec2::new(1.0, 1.0),
        LinearRgba::WHITE,
    );
}
