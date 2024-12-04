use bevy::{
    color::palettes::css::{DARK_GREEN, TAN},
    core_pipeline::auto_exposure::AutoExposurePlugin,
    prelude::*,
};

use bevy_map_camera::{CameraPerspectiveState, LookTransform, MapCamera, MapCameraPlugin};

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
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10., 10.))),
        MeshMaterial3d(materials.add(Color::from(DARK_GREEN))),
    ));

    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::from(TAN))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

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
                y: 4.5,
                z: 15.0,
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
                Text("Press Right Mouse Button to pan".to_owned()),
                Node {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..Default::default()
                },
                Label,
            ));

            parent.spawn((
                Text(ROTATION_ENABLED.to_owned()),
                Node {
                    margin: UiRect::all(Val::Px(5.0)),
                    align_self: AlignSelf::FlexStart,
                    ..default()
                },
                Label,
                RotationLabel,
            ));

            parent.spawn((
                Text("Press Middle Mouse Button to toggle projection".to_owned()),
                Node {
                    margin: UiRect::all(Val::Px(5.0)),
                    align_self: AlignSelf::FlexStart,
                    ..default()
                },
                Label,
            ));
        });

    commands.spawn((
        Text(PERSPECTIVE.to_owned()),
        Node {
            margin: UiRect::all(Val::Px(5.0)),
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            ..default()
        },
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
        projection_text.single_mut().0 = String::from(p_text);

        //Rotation text
        rotation_text.single_mut().0 = String::from(r_text);
    }
}
