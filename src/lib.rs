#![doc = include_str!("../README.md")]

pub mod controller;
pub mod inputs;
pub mod look_angles;
pub mod look_transform;

use bevy_app::prelude::*;
use bevy_core_pipeline::prelude::*;
use bevy_ecs::prelude::*;
use bevy_input::InputSystem;
use bevy_render::prelude::*;
use bevy_transform::components::Transform;

// re-exports
pub use controller::{CameraController, CameraControllerSettings};
pub use look_transform::LookTransform;
use look_transform::Smoother;

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

        // logic for camera input (buttons and inputdevices)
        app.add_plugins(controller::CameraControllerPlugin);

        #[cfg(feature = "bevy_easings")]
        app.add_systems(
            Update,
            bevy_easings::custom_ease_system::<(), LookTransform>.in_set(CameraChange::Before),
        );
        #[cfg(feature = "bevy_tweening")]
        {
            if !app.is_plugin_added::<bevy_tweening::TweeningPlugin>() {
                app.add_plugins(bevy_tweening::TweeningPlugin);
            }
            app.add_systems(
                Update,
                bevy_tweening::component_animator_system::<LookTransform>
                    .in_set(CameraChange::Before)
                    .in_set(bevy_tweening::AnimationSystem::AnimationUpdate),
            );
        }
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

#[derive(Component)]
#[require(CameraController, Camera3d, LookTransform, Camera = default_camera())]
pub struct MapCamera;

fn default_camera() -> Camera {
    Camera {
        msaa_writeback: false,
        ..Default::default()
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum CameraChange {
    /// Systems that should run before any changes to the camera transform are made
    Before,
    /// Systems that should run after any changes to the camera transform are made
    After,
}
