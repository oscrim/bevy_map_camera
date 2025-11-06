#![doc = include_str!("../README.md")]
#[deny(warnings)]
pub mod controller;
pub mod inputs;
pub mod look_angles;
pub mod look_transform;

use bevy_app::prelude::*;
use bevy_camera::{Camera, Camera3d};
use bevy_ecs::prelude::*;
use bevy_input::InputSystems;
use bevy_transform::components::Transform;

// re-exports
pub use controller::{CameraController, CameraControllerSettings};
pub use look_transform::LookTransform;

/// Orbital camera plugin
#[derive(Clone, Copy)]
pub struct MapCameraPlugin;

impl Plugin for MapCameraPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            PreUpdate,
            (CameraChange::Before, CameraChange::After)
                .chain()
                .after(InputSystems),
        );

        app.add_systems(
            PreUpdate,
            look_transform_system
                .after(CameraChange::Before)
                .before(CameraChange::After),
        );

        // logic for camera input (buttons and inputdevices)
        app.add_plugins(controller::CameraControllerPlugin);

        #[cfg(feature = "bevy_easings")]
        app.add_systems(
            PreUpdate,
            bevy_easings::custom_ease_system::<(), LookTransform>.in_set(CameraChange::Before),
        );
        #[cfg(feature = "bevy_tweening")]
        if !app.is_plugin_added::<bevy_tweening::TweeningPlugin>() {
            app.add_plugins(bevy_tweening::TweeningPlugin);
        }
    }
}

fn look_transform_system(mut lts: Query<(&LookTransform, &mut Transform)>) {
    lts.iter_mut()
        .for_each(|(&look_transform, mut scene_transform)| {
            *scene_transform = look_transform.into();
        });
}

#[derive(Component)]
#[require(CameraController, Camera3d, LookTransform, Camera = default_camera())]
pub struct MapCamera;

fn default_camera() -> Camera {
    Camera {
        msaa_writeback: false,
        ..Default::default()
    }
}

/// Runs in the PreUpdate schedule
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum CameraChange {
    /// Systems that should run before any changes to the camera transform are made
    Before,
    /// Systems that should run after any changes to the camera transform are made
    After,
}
