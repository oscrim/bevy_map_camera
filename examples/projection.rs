use bevy::prelude::*;

use bevy_map_cam::{CameraBundle, CameraPerspectiveState, LookTransform, MapCameraPlugin};

const ROTATION_ENABLED: &str = "Press Left Mouse Button to rotate";
const ROTATION_DISABLED: &str = "Rotation Disabled";

const PERSPECTIVE: &str = "Perspective Projection";
const ORTHOGRAPHIC: &str = "Orthographic Projection";

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(MapCameraPlugin::default());

    app.add_systems(Startup, setup);
    app.add_systems(Update, (trigger_perspective_change, projection_changed));

    app.run();
}

#[derive(Component)]
struct ProjectionLabel;

#[derive(Component)]
struct RotationLabel;

fn setup(
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
                    "Press Right Mouse Button to pan",
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
                    ROTATION_ENABLED,
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
                RotationLabel,
            ));

            parent.spawn((
                TextBundle::from_section(
                    "Press Middle Mouse Button to toggle projection",
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

    commands.spawn((
        TextBundle::from_section(
            PERSPECTIVE,
            TextStyle {
                font_size: 30.0,
                color: Color::BLACK,
                ..Default::default()
            },
        )
        .with_style(Style {
            margin: UiRect::all(Val::Px(5.0)),
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            ..default()
        }),
        Label,
        ProjectionLabel,
    ));
}

fn trigger_perspective_change(
    button: Res<ButtonInput<MouseButton>>,
    current_state: Res<State<CameraPerspectiveState>>,
    mut next_state: ResMut<NextState<CameraPerspectiveState>>,
) {
    if button.just_pressed(MouseButton::Middle) {
        match current_state.get() {
            CameraPerspectiveState::Orthographic => {
                next_state.set(CameraPerspectiveState::Perspective)
            }
            CameraPerspectiveState::Perspective => {
                next_state.set(CameraPerspectiveState::Orthographic)
            }
        };
    }
}

fn projection_changed(
    projection_query: Query<&Projection, (With<LookTransform>, Changed<Projection>, Without<Text>)>,
    mut projection_text: Query<&mut Text, (With<ProjectionLabel>, Without<RotationLabel>)>,
    mut rotation_text: Query<&mut Text, (With<RotationLabel>, Without<ProjectionLabel>)>,
) {
    if let Ok(projection) = projection_query.get_single() {
        let (p_text, r_text) = match projection {
            Projection::Perspective(_) => (PERSPECTIVE, ROTATION_ENABLED),
            Projection::Orthographic(_) => (ORTHOGRAPHIC, ROTATION_DISABLED),
        };

        //Projection text
        {
            let mut text = projection_text.single_mut();

            let mut section = text.sections[0].clone();

            section.value = String::from(p_text);

            text.sections = vec![section];
        }

        //Rotation text
        {
            let mut text = rotation_text.single_mut();

            let mut section = text.sections[0].clone();

            section.value = String::from(r_text);

            text.sections = vec![section];
        }
    }
}
