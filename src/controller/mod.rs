mod mouse;
mod mouse_input;
mod resources;
mod touch;
mod touch_inputs;

use std::f32::consts::PI;

use bevy::{prelude::*, window::WindowFocused};

use crate::{
    inputs::InputButton, look_angles::LookAngles, CameraChange, CameraPerspectiveState,
    LookTransform,
};

pub use resources::CameraControllerButtons;

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
pub struct CameraController {
    pub enabled: bool,
    pub pixels_per_line: f32,
    pub smoothing_weight: f32,
    /// The height of the grab plane that the camera moves along
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
    fn lerp(&mut self, target: &mut dyn bevy_tweening::Targetable<CameraController>, ratio: f32) {
        target.grab_height = self.start.lerp(self.end, ratio);
    }
}

#[derive(Event)]
enum ControlEvent {
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
        app.init_resource::<CameraControllerSettings>();

        app.add_event::<ControlEvent>();

        app.add_plugins(mouse::MouseController);
        app.add_plugins(touch::TouchInputPlugin);

        app.add_systems(
            Update,
            (
                (
                    control_system.run_if(on_event::<ControlEvent>()),
                    update_height,
                )
                    .chain()
                    .after(CameraChange::Before)
                    .before(super::look_transform_system),
                clear_inputs_on_focus.after(CameraChange::After),
            ),
        );

        #[cfg(feature = "bevy_tweening")]
        app.add_systems(
            Update,
            bevy_tweening::component_animator_system::<CameraController>
                .in_set(CameraChange::Before),
        );
    }
}

fn update_height(
    mut camera: Query<(&mut LookTransform, &CameraController), Changed<CameraController>>,
) {
    let Ok((mut transform, controller)) = camera.get_single_mut() else {
        return;
    };

    if !controller.enabled {
        return;
    }

    let y_diff = controller.grab_height - transform.target.y;
    transform.target.y = controller.grab_height;

    transform.eye.y += y_diff;
}

fn control_system(
    mut events: EventReader<ControlEvent>,
    mut camera: Query<(&mut Projection, &mut LookTransform, &CameraController)>,
    camera_state: Res<State<CameraPerspectiveState>>,
    settings: Res<CameraControllerSettings>,
) {
    let (mut projection, mut transform, controller) = camera.single_mut();

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
            ControlEvent::Orbit(delta) => {
                look_angles.add_yaw(-delta.x);
                look_angles.add_pitch(delta.y);

                if look_angles.get_pitch() < settings.minimum_pitch {
                    look_angles.set_pitch(settings.minimum_pitch)
                }
            }
            ControlEvent::TranslateTarget(delta) => {
                transform.target += *delta;
            }
            ControlEvent::Zoom {
                zoom_scalar,
                zoom_target,
            } => {
                radius_scalar *= zoom_scalar;

                let new_radius = radius_scalar * radius;

                if new_radius < settings.minimum_zoom || new_radius > settings.maximum_zoom {
                    continue;
                }

                let mut dir = transform.target - *zoom_target;
                dir.y = 0.0;

                transform.target -= dir * (1. - *zoom_scalar);
            }
        }
    }

    look_angles.assert_not_looking_up();

    let new_radius = (radius_scalar * radius)
        .min(settings.maximum_zoom)
        .max(settings.minimum_zoom);

    transform.target.y = controller.grab_height;

    transform.eye = transform.target + new_radius * look_angles.unit_vector();

    // Add one to make sure the eye is inside the grab plane
    transform.eye.y = transform.eye.y.max(controller.grab_height + 1.0);

    if let CameraPerspectiveState::Orthographic = camera_state.get() {
        if let Projection::Orthographic(o) = &mut *projection {
            o.scale *= new_radius / radius;
        }
    }
}

/// On the web the input is not cleared if the focus is quickly switched
/// for example using a shortcut to change tab.
fn clear_inputs_on_focus(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut event_reader: EventReader<WindowFocused>,
) {
    if let Some(ev) = event_reader.read().last() {
        if ev.focused == true {
            keys.release_all();
            mouse.release_all();
        }
    }
}

fn ray_from_screenspace(
    cursor_pos_screen: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    window: &Window,
) -> Option<Ray3d> {
    let mut viewport_pos = cursor_pos_screen;
    if let Some(viewport) = &camera.viewport {
        viewport_pos -= viewport.physical_position.as_vec2() / window.scale_factor();
    }
    camera
        .viewport_to_world(camera_transform, viewport_pos)
        .map(Ray3d::from)
}
