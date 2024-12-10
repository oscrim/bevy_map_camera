use bevy::prelude::*;
use bevy_window::{PrimaryWindow, SystemCursorIcon, Window};
use bevy_winit::cursor::CursorIcon;

use super::ray_from_screenspace;
use crate::{look_transform::Smoother, CameraProjectionState};

use super::{
    mouse_input::MouseKeyboardInputs, CameraController, CameraControllerSettings, ControlEvent,
};
use crate::{inputs::Inputs, CameraChange, LookTransform};

pub(super) struct MouseController;

impl Plugin for MouseController {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (zoom_orbit_camera, pan_rotate_orbit_camera, grab_pan)
                .chain()
                .in_set(CameraChange::Before),
        );
    }
}

/// Handles the panning of the camera
fn pan_rotate_orbit_camera(
    settings: Res<CameraControllerSettings>,
    camstate: Res<State<CameraProjectionState>>,
    mut camera_writer: EventWriter<ControlEvent>,
    mut mouse_inputs: MouseKeyboardInputs,
) {
    let mut rotation_move = mouse_inputs.mouse_drag(&settings.buttons.rotate);

    if let (Some(alt), true) = (&settings.buttons.rotate_alt, rotation_move.is_none()) {
        rotation_move = mouse_inputs.mouse_drag(alt);
    }

    // only rotate in perspective state
    if let (Some(rotation_move), CameraProjectionState::Perspective) =
        (rotation_move, *camstate.get())
    {
        camera_writer.send(ControlEvent::Orbit(
            rotation_move * settings.mouse_rotation_sensitivity_modifier,
        ));
    }
}

/// Handles the zooming of the orbital camera
fn zoom_orbit_camera(
    query: Query<(&CameraController, &Camera, &GlobalTransform, &LookTransform)>,
    settings: Res<CameraControllerSettings>,
    main_window: Query<&Window, With<PrimaryWindow>>,
    mut mouse_inputs: MouseKeyboardInputs,
    mut camera_writer: EventWriter<ControlEvent>,
) {
    let (controller, camera, camera_gt, camera_lt) = query.single();
    let window = main_window.single();

    let scroll_sensitivity = settings.mouse_zoom_sensitivity_modifier;

    let Some(scalar) = mouse_inputs.scroll_scalar(controller.pixels_per_line, scroll_sensitivity)
    else {
        return;
    };

    let Some(mouse_pos) = window.cursor_position() else {
        camera_writer.send(ControlEvent::Zoom {
            zoom_scalar: scalar,
            zoom_target: camera_lt.target,
        });

        return;
    };

    let Ok(ray) = ray_from_screenspace(mouse_pos, camera, camera_gt, window) else {
        camera_writer.send(ControlEvent::Zoom {
            zoom_scalar: scalar,
            zoom_target: camera_lt.target,
        });

        return;
    };

    let Some(target_distance) = ray.intersect_plane(
        Vec3::Y * controller.grab_height,
        InfinitePlane3d { normal: Dir3::Y },
    ) else {
        return;
    };

    let target = ray.get_point(target_distance);

    camera_writer.send(ControlEvent::Zoom {
        zoom_scalar: scalar,
        zoom_target: target,
    });
}

fn grab_pan(
    mut commands: Commands,
    mut cam_q: Query<(
        &GlobalTransform,
        &LookTransform,
        &Camera,
        &mut Smoother,
        &CameraController,
    )>,
    settings: Res<CameraControllerSettings>,
    inputs: Inputs,
    mut first_ray_hit: Local<Option<Vec3>>,
    mut primary_window_q: Query<(Entity, Option<&mut CursorIcon>, &Window), With<PrimaryWindow>>,
    mut camera_writer: EventWriter<ControlEvent>,
    mut saved_smoother_weight: Local<f32>,
) {
    let (camera_gt, look_transform, camera, mut smoother, controller) = cam_q.single_mut();
    let (window_entity, mut cursor_icon, primary_window) = primary_window_q.single_mut();
    let drag_buttons = &settings.buttons.pan;

    if inputs.multi_just_pressed(drag_buttons) {
        if let Some(mouse_pos) = primary_window.cursor_position() {
            if let Ok(ray) = ray_from_screenspace(mouse_pos, camera, camera_gt, primary_window) {
                let Some(target_distance) = ray.intersect_plane(
                    Vec3::Y * controller.grab_height,
                    InfinitePlane3d { normal: Dir3::Y },
                ) else {
                    log::error!("Grab pan intersection did not intersect with Grab plane");
                    return;
                };

                if let Some(icon) = cursor_icon.as_mut() {
                    icon.set_if_neq(CursorIcon::System(SystemCursorIcon::Grabbing));
                } else if let Some(mut ecmd) = commands.get_entity(window_entity) {
                    ecmd.insert(CursorIcon::System(SystemCursorIcon::Grabbing));
                }

                *saved_smoother_weight = smoother.lag_weight;
                smoother.lag_weight = 0.1;

                *first_ray_hit = Some(ray.get_point(target_distance));
            }
        }
    }

    if inputs.multi_just_released(drag_buttons) {
        smoother.lag_weight = *saved_smoother_weight;
        *first_ray_hit = None;
        if let Some(icon) = cursor_icon.as_mut() {
            icon.set_if_neq(CursorIcon::System(SystemCursorIcon::Default));
        }
    }

    if inputs.multi_pressed(drag_buttons) {
        let Some(first_hit) = *first_ray_hit else {
            //Grab pan pressed without first ray hit, return
            return;
        };

        if let Some(mouse_pos) = primary_window.cursor_position() {
            if let Ok(ray) = ray_from_screenspace(mouse_pos, camera, camera_gt, primary_window) {
                let Some(target_distance) = ray.intersect_plane(
                    Vec3::Y * controller.grab_height,
                    InfinitePlane3d { normal: Dir3::Y },
                ) else {
                    log::error!("Grab pan intersection did not intersect with Grab plane");
                    return;
                };
                let new_hit = ray.get_point(target_distance);

                // Compensate for look transform smoothing to prevent jittering
                let smoothing_target_diff = if let Some(smoothing_transform) = smoother.lerp_tfm {
                    look_transform.target - smoothing_transform.target
                } else {
                    Vec3::ZERO
                };

                let first_hit_diff = first_hit - new_hit - smoothing_target_diff;

                camera_writer.send(ControlEvent::TranslateTarget(first_hit_diff));
            }
        }
    }
}
