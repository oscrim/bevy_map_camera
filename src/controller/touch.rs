use super::{ray_from_screenspace, CameraController};
use crate::{look_transform::Smoother, CameraProjectionState};
use bevy::prelude::*;
use bevy_window::{PrimaryWindow, Window};

use crate::{CameraChange, LookTransform};

use super::{
    touch_inputs::{Pinch, TouchInputSettings, TouchInputs},
    CameraControllerSettings, ControlEvent,
};

pub(super) struct TouchInputPlugin;

impl Plugin for TouchInputPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TouchInputSettings>();
        app.init_resource::<TouchInputSettings>();
        app.add_systems(
            Update,
            (
                zoom_orbit_camera,
                grab_pan,
                rotate_orbit_camera.run_if(in_state(CameraProjectionState::Perspective)),
            )
                .in_set(CameraChange::Before),
        );
    }
}

/// Handles the zooming of the orbital camera
fn zoom_orbit_camera(
    mut touches: TouchInputs,
    query: Query<(&Camera, &GlobalTransform, &LookTransform, &CameraController)>,
    main_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<CameraControllerSettings>,
    mut camera_writer: EventWriter<ControlEvent>,
) {
    // Get the deltas of the two touches
    let Some(Pinch {
        distance_delta,
        middle,
    }) = touches.get_pinch()
    else {
        return;
    };

    let Ok((camera, camera_gt, camera_lt, controller)) = query.get_single() else {
        return;
    };
    let window = main_window.single();

    let scalar = 1.0 - distance_delta * settings.touch_zoom_sensitivity_modifier;

    if scalar == 1.0 {
        return;
    }

    let Ok(ray) = ray_from_screenspace(middle, camera, camera_gt, window) else {
        camera_writer.send(ControlEvent::Zoom {
            zoom_scalar: scalar,
            zoom_target: camera_lt.target,
        });
        error!("Unable to create ray from screenspace");
        return;
    };

    let Some(target_distance) = ray.intersect_plane(
        Vec3::Y * controller.grab_height,
        InfinitePlane3d { normal: Dir3::Y },
    ) else {
        warn!("Cursor click did not intersect with Grab plane");
        return;
    };

    let target = ray.get_point(target_distance);

    camera_writer.send(ControlEvent::Zoom {
        zoom_scalar: scalar,
        zoom_target: target,
    });
}

/// Handles the rotation of the orbital camera, dont run in orthographic
fn rotate_orbit_camera(
    mut touches: TouchInputs,
    settings: Res<CameraControllerSettings>,
    mut camera_writer: EventWriter<ControlEvent>,
) {
    let Some(rotation_move) = touches.get_two_touch_drag() else {
        return;
    };
    camera_writer.send(ControlEvent::Orbit(
        rotation_move * settings.touch_rotation_sensitivity_modifier,
    ));
}

fn grab_pan(
    mut cam_q: Query<(
        &GlobalTransform,
        &LookTransform,
        &Camera,
        &mut Smoother,
        &CameraController,
    )>,
    mut inputs: TouchInputs,
    mut first_ray_hit: Local<Option<Vec3>>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
    mut camera_writer: EventWriter<ControlEvent>,
    mut saved_smoother_weight: Local<f32>,
    mut over_threshold: Local<bool>,
    mut first_screen_touch: Local<Option<Vec2>>,
) {
    let Ok((camera_gt, look_transform, camera, mut smoother, controller)) = cam_q.get_single_mut()
    else {
        return;
    };
    let primary_window = primary_window_q.single();

    if let Some(touch_pos) = inputs.get_one_touch_just_press() {
        if let Ok(ray) = ray_from_screenspace(touch_pos, camera, camera_gt, primary_window) {
            let Some(target_distance) = ray.intersect_plane(
                Vec3::Y * controller.grab_height,
                InfinitePlane3d { normal: Dir3::Y },
            ) else {
                warn!("Grab pan intersection did not intersect with Grab plane");
                return;
            };

            *saved_smoother_weight = smoother.lag_weight;
            smoother.lag_weight = 0.1;

            *first_ray_hit = Some(ray.get_point(target_distance));
            *over_threshold = false;
            *first_screen_touch = Some(touch_pos);
        }
    }

    if inputs.one_just_released() {
        smoother.lag_weight = *saved_smoother_weight;
        *first_ray_hit = None;
        *first_screen_touch = None;
    }

    if let (Some(touch_pos), Some(first_hit), Some(screen_touch)) = (
        inputs.get_one_touch_drag_pos(),
        *first_ray_hit,
        *first_screen_touch,
    ) {
        if let Ok(ray) = ray_from_screenspace(touch_pos, camera, camera_gt, primary_window) {
            let Some(target_distance) = ray.intersect_plane(
                Vec3::Y * controller.grab_height,
                InfinitePlane3d { normal: Dir3::Y },
            ) else {
                warn!("Grab pan intersection did not intersect with Grab plane");
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

            if touch_pos.distance(screen_touch) > 3.0 || *over_threshold {
                *over_threshold = true;
                camera_writer.send(ControlEvent::TranslateTarget(first_hit_diff));
            }
        }
    }
}
