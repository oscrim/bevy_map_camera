use bevy::{
    color::palettes::css::{DARK_GREEN, TAN},
    core_pipeline::auto_exposure::AutoExposurePlugin,
    prelude::*,
};

use bevy_map_camera::{CameraPerspectiveState, LookTransform, MapCameraBundle, MapCameraPlugin};

const ROTATION_ENABLED: &str = "Press Left Mouse Button to rotate";
const ROTATION_DISABLED: &str = "Rotation Disabled";

const PERSPECTIVE: &str = "Perspective Projection";
const ORTHOGRAPHIC: &str = "Orthographic Projection";

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, AutoExposurePlugin, MapCameraPlugin));

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
        mesh: meshes.add(Plane3d::default().mesh().size(10., 10.)),
        material: materials.add(Color::from(DARK_GREEN)),
        ..Default::default()
    });

    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::default()),
        material: materials.add(Color::from(TAN)),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });

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
            y: 4.5,
            z: 15.0,
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
                TextBundle::from_section("Press Right Mouse Button to pan", TextStyle::default())
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(5.0)),
                        ..Default::default()
                    }),
                Label,
            ));

            parent.spawn((
                TextBundle::from_section(ROTATION_ENABLED, TextStyle::default()).with_style(
                    Style {
                        margin: UiRect::all(Val::Px(5.0)),
                        align_self: AlignSelf::FlexStart,
                        ..default()
                    },
                ),
                Label,
                RotationLabel,
            ));

            parent.spawn((
                TextBundle::from_section(
                    "Press Middle Mouse Button to toggle projection",
                    TextStyle::default(),
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
        TextBundle::from_section(PERSPECTIVE, TextStyle::default()).with_style(Style {
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
