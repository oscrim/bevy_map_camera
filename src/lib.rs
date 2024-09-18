#![doc = include_str!("../README.md")]

use bevy::prelude::*;

pub mod controller;
pub mod inputs;
pub mod look_angles;
pub mod look_transform;
pub mod projection;

use bevy_input::InputSystem;
// re-exports
pub use controller::{CameraController, CameraControllerSettings};
pub use look_transform::LookTransform;
use look_transform::{LookTransformBundle, Smoother};

/// Orbital camera plugin
#[derive(Clone, Copy)]
pub struct MapCameraPlugin;

impl Plugin for MapCameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<LookTransform>();

        app.configure_sets(
            Update,
            (CameraChange::Before, CameraChange::After)
                .chain()
                .after(InputSystem),
        );

        app.add_systems(
            Update,
            look_transform_system
                .after(CameraChange::Before)
                .before(CameraChange::After),
        );

        app.init_state::<CameraPerspectiveState>();

        // logic for camera input (buttons and inputdevices)
        app.add_plugins((
            controller::CameraControllerPlugin,
            projection::ChangeProjectionPlugin,
        ));

        #[cfg(feature = "bevy_easings")]
        app.add_systems(
            Update,
            bevy_easings::custom_ease_system::<LookTransform>.in_set(CameraChange::Before),
        );
    }
}

fn look_transform_system(
    mut cameras: Query<(&LookTransform, &mut Transform, Option<&mut Smoother>)>,
) {
    for (look_transform, mut scene_transform, smoother) in cameras.iter_mut() {
        match smoother {
            Some(mut s) if s.enabled => {
                *scene_transform = s.smooth_transform(look_transform).into();
            }
            _ => (),
        };
    }
}

#[derive(Bundle)]
pub struct MapCameraBundle {
    pub camera_3d: Camera3dBundle,
    pub controller: CameraController,
    pub look_transform: LookTransformBundle,
}

impl MapCameraBundle {
    pub fn new_with_transform(look_transform: LookTransform) -> Self {
        let transform = Transform::from_translation(look_transform.eye)
            .looking_at(look_transform.target, Vec3::Y);

        let mut bundle = Self::default();

        bundle.camera_3d.transform = transform;
        bundle.look_transform.transform = look_transform;

        bundle
    }
}

impl Default for MapCameraBundle {
    fn default() -> Self {
        let look_transform = LookTransform::new(Vec3::ONE * 5.0, Vec3::ZERO, Vec3::Y);

        let transform = Transform::from_translation(look_transform.eye)
            .looking_at(look_transform.target, Vec3::Y);

        Self {
            camera_3d: Camera3dBundle {
                camera: Camera {
                    msaa_writeback: false,
                    ..Default::default()
                },
                transform,
                ..Default::default()
            },
            controller: Default::default(),
            look_transform: LookTransformBundle {
                transform: look_transform,
                smoother: Default::default(),
            },
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum CameraChange {
    /// Systems that should run before any changes to the camera transform are made
    Before,
    /// Systems that should run after any changes to the camera transform are made
    After,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default, States)]
pub enum CameraPerspectiveState {
    #[default]
    Perspective,
    Orthographic,
}
