use std::f32::consts::PI;
use std::time::Duration;

use bevy::color::palettes::css::{DARK_GREEN, TAN};
use bevy::core_pipeline::auto_exposure::AutoExposurePlugin;
use bevy::prelude::*;

use bevy_input::common_conditions::input_just_pressed;
use bevy_map_camera::controller::GrabHeightLens;
use bevy_map_camera::{
    CameraController, CameraControllerSettings, LookTransform, MapCameraBundle, MapCameraPlugin,
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

            parent.spawn((
                TextBundle::from_section("Press ArrowUp to increase height", TextStyle::default())
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(5.0)),
                        ..default()
                    }),
                Label,
            ));

            parent.spawn((
                TextBundle::from_section(
                    "Press ArrowDown to decrease height",
                    TextStyle::default(),
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                }),
                Label,
            ));

            parent.spawn((
                TextBundle::from_section(
                    "Press Space to toggle height animation",
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
        EaseMethod::Linear,
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
    let position = Vec3::NEG_Z * controller_query.single().grab_height;

    gizmos.grid(
        position,
        Quat::from_axis_angle(Vec3::X, PI / 2.0),
        UVec2::new(10, 10),
        Vec2::new(1.0, 1.0),
        LinearRgba::WHITE,
    );
}
