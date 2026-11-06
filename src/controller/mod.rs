mod mouse;
mod mouse_input;
mod resources;
mod surface;
mod touch;
mod touch_inputs;

use std::f32::consts::PI;

use bevy_app::{App, Plugin, PreUpdate};
use bevy_camera::{Camera, ViewportConversionError, primitives::Aabb};
use bevy_ecs::prelude::*;

use bevy_math::{Dir3, Ray3d, Vec2, Vec3};
use bevy_reflect::prelude::*;
use bevy_transform::components::GlobalTransform;
use bevy_window::Window;

use crate::{CameraChange, LookTransform, inputs::InputButton, look_angles::LookAngles};

pub use resources::CameraControllerButtons;
pub use surface::GrabSurface;

use surface::{GrabSurfaces, SurfaceFollow};

#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct CameraControllerSettings {
    /// Enabled by default
    pub touch_enabled: bool,
    pub mouse_zoom_sensitivity_modifier: f32,
    pub mouse_rotation_sensitivity_modifier: f32,
    pub touch_zoom_sensitivity_modifier: f32,
    pub touch_rotation_sensitivity_modifier: f32,
    pub touch_translation_sensitivity_modifier: f32,
    /// In radians
    pub minimum_pitch: f32,
    pub minimum_zoom: f32,
    pub maximum_zoom: f32,
    /// Buttons to use when controlling the camera with a mouse (or some touchpads)
    pub buttons: CameraControllerButtons,
}

impl Default for CameraControllerSettings {
    fn default() -> Self {
        Self {
            touch_enabled: true,
            mouse_zoom_sensitivity_modifier: 0.06,
            mouse_rotation_sensitivity_modifier: 0.00544,
            touch_rotation_sensitivity_modifier: 0.008,
            touch_zoom_sensitivity_modifier: 0.008,
            touch_translation_sensitivity_modifier: 0.02,
            minimum_pitch: 25.0 * PI / 180.0,
            minimum_zoom: 1.5,
            maximum_zoom: 1_000.0,
            buttons: CameraControllerButtons::default(),
        }
    }
}

impl CameraControllerSettings {
    pub fn with_pan_button(mut self, btn: Vec<InputButton>) -> Self {
        self.buttons.pan = btn;
        self
    }
    pub fn with_rotate_button(mut self, btn: Vec<InputButton>) -> Self {
        self.buttons.rotate = btn;
        self
    }
}

/// A 3rd person camera that orbits around the target.
#[derive(Clone, Component, Copy, Debug, Reflect)]
#[reflect(Component, Default, Debug)]
#[require(SurfaceFollow)]
pub struct CameraController {
    pub enabled: bool,
    pub pixels_per_line: f32,
    pub smoothing_weight: f32,
    /// The height of the grab plane that the camera moves along.
    ///
    /// Ignored while any [`GrabSurface`] exists.
    pub grab_height: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            pixels_per_line: 53.0,
            smoothing_weight: 0.8,
            grab_height: 0.0,
        }
    }
}

#[cfg(feature = "bevy_tweening")]
pub struct GrabHeightLens {
    pub start: f32,
    pub end: f32,
}

#[cfg(feature = "bevy_tweening")]
impl bevy_tweening::Lens<CameraController> for GrabHeightLens {
    fn lerp(&mut self, mut target: Mut<CameraController>, ratio: f32) {
        use bevy_math::FloatExt;
        target.grab_height = self.start.lerp(self.end, ratio);
    }
}

#[derive(Message)]
pub enum ControlMessage {
    Orbit(Vec2),
    /// Translation Delta
    TranslateTarget(Vec3),
    Zoom {
        zoom_scalar: f32,
        zoom_target: Vec3,
    },
}

pub(crate) struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraController>();
        app.register_type::<CameraControllerSettings>();
        app.register_type::<GrabSurface>();
        app.init_resource::<CameraControllerSettings>();

        app.add_message::<ControlMessage>();

        app.add_plugins(mouse::MouseController);
        app.add_plugins(touch::TouchInputPlugin);

        app.add_systems(
            PreUpdate,
            (
                // Before control_system, so a removed surface is handled before it pins the height
                update_height,
                control_system.run_if(on_message::<ControlMessage>),
                snap_target_to_surface,
            )
                .chain()
                .after(CameraChange::Before)
                .before(super::look_transform_system),
        );
    }
}

fn update_height(
    camera: Single<(
        &mut LookTransform,
        Ref<CameraController>,
        &mut SurfaceFollow,
    )>,
    settings: Res<CameraControllerSettings>,
    surfaces: GrabSurfaces,
    mut was_active: Local<bool>,
) {
    let (mut transform, controller, mut follow) = camera.into_inner();

    // Checked before tracking the surfaces, so a removal while disabled is handled on re-enable
    if !controller.enabled {
        return;
    }

    let active = surfaces.is_active();
    let surfaces_removed = std::mem::replace(&mut *was_active, active) && !active;

    if active {
        return;
    }

    if surfaces_removed {
        // Surfaces added later should be followed again
        follow.set_if_neq(SurfaceFollow::default());

        // Slide the target along the look direction onto the grab plane, so the view stays put
        if let Ok(look_direction) = Dir3::new(transform.target - transform.eye)
            && let Some(point) = surface::intersect_grab_height(
                Ray3d::new(transform.eye, look_direction),
                &controller,
            )
            && (settings.minimum_zoom..=settings.maximum_zoom)
                .contains(&point.distance(transform.eye))
        {
            transform.target = point;
            return;
        }
    } else if !controller.is_changed() {
        return;
    }

    let y_diff = controller.grab_height - transform.target.y;
    if y_diff == 0.0 {
        return;
    }
    transform.target.y = controller.grab_height;

    transform.eye.y += y_diff;
}

fn control_system(
    mut events: MessageReader<ControlMessage>,
    camera: Single<(&mut LookTransform, &CameraController)>,
    settings: Res<CameraControllerSettings>,
    surfaces: GrabSurfaces,
) {
    let (mut transform, controller) = camera.into_inner();
    // With grab surfaces the target height is set by `snap_target_to_surface` instead
    let use_grab_height = !surfaces.is_active();

    if !controller.enabled {
        // Read all events to mark them as read
        for _ev in events.read() {}
        return;
    }

    let mut look_angles = LookAngles::from_vector(
        -transform
            .look_direction()
            .expect("Failed to normalize look direction"),
    );

    let mut radius_scalar = 1.0;
    let radius = transform.radius();

    for event in events.read() {
        match event {
            ControlMessage::Orbit(delta) => {
                look_angles.add_yaw(-delta.x);
                look_angles.add_pitch(delta.y);

                if look_angles.get_pitch() < settings.minimum_pitch {
                    look_angles.set_pitch(settings.minimum_pitch)
                }
            }
            ControlMessage::TranslateTarget(delta) => {
                transform.target += *delta;
            }
            ControlMessage::Zoom {
                zoom_scalar,
                zoom_target,
            } => {
                radius_scalar *= zoom_scalar;

                let new_radius = radius_scalar * radius;

                if new_radius < settings.minimum_zoom || new_radius > settings.maximum_zoom {
                    continue;
                }

                let mut dir = transform.target - *zoom_target;
                if use_grab_height {
                    // The target and zoom point are both on the grab plane
                    dir.y = 0.0;
                }

                transform.target -= dir * (1. - *zoom_scalar);
            }
        }
    }

    look_angles.assert_not_looking_up();

    let new_radius = (radius_scalar * radius)
        .min(settings.maximum_zoom)
        .max(settings.minimum_zoom);

    if use_grab_height {
        transform.target.y = controller.grab_height;
    }

    transform.eye = transform.target + new_radius * look_angles.unit_vector();

    // Make sure the eye stays above the grab plane
    transform.eye.y = transform.eye.y.max(transform.target.y + 0.1);
}

/// A hittable [`GrabSurface`] that was added or moved.
type ChangedSurface = (
    surface::HittableSurface,
    Or<(
        Changed<GrabSurface>,
        Changed<GlobalTransform>,
        Changed<Aabb>,
    )>,
);

/// Moves the camera target along the look direction onto the [`GrabSurface`] in the centre
/// of the view. The eye stays put, so the view itself does not change.
fn snap_target_to_surface(
    camera: Single<(
        &mut LookTransform,
        Ref<CameraController>,
        Ref<SurfaceFollow>,
    )>,
    settings: Res<CameraControllerSettings>,
    mut surfaces: GrabSurfaces,
    changed_surfaces: Query<(), ChangedSurface>,
    mut removed_surfaces: RemovedComponents<GrabSurface>,
) {
    let (mut transform, controller, follow) = camera.into_inner();
    // Always drain the removals so old ones don't trigger a cast later
    let surface_removed = removed_surfaces.read().count() > 0;

    if !controller.enabled || follow.floating || !surfaces.is_active() {
        return;
    }

    // Only ray cast when the camera or a surface changed. A controller change covers being
    // re-enabled after surface changes that happened while disabled.
    if !transform.is_changed()
        && !follow.is_changed()
        && !controller.is_changed()
        && !surface_removed
        && changed_surfaces.is_empty()
    {
        return;
    }

    let Ok(look_direction) = Dir3::new(transform.target - transform.eye) else {
        return;
    };

    let Some(hit) = surfaces.cast(Ray3d::new(transform.eye, look_direction)) else {
        return;
    };

    // Keep the target where it is rather than break the zoom limits, which would move the eye
    let distance = hit.distance(transform.eye);
    if distance < settings.minimum_zoom || distance > settings.maximum_zoom {
        return;
    }

    if hit.distance_squared(transform.target) > 1e-8 {
        transform.target = hit;
    }
}

fn ray_from_screenspace(
    cursor_pos_screen: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    window: &Window,
) -> Result<Ray3d, ViewportConversionError> {
    let mut viewport_pos = cursor_pos_screen;
    if let Some(viewport) = &camera.viewport {
        viewport_pos -= viewport.physical_position.as_vec2() / window.scale_factor();
    }
    camera.viewport_to_world(camera_transform, viewport_pos)
}
