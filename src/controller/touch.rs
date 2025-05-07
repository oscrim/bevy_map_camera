use super::{ray_from_screenspace, CameraController};
use crate::{look_transform::Smoother, CameraProjectionState};
use bevy::{
    picking::{
        backend::ray::{RayId, RayMap},
        pointer::PointerId,
    },
    prelude::*,
    utils::HashMap,
};
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
    mut cam_q: Query<(Entity, &LookTransform, &mut Smoother, &CameraController), With<Camera>>,
    mut inputs: TouchInputs,
    touches: Res<Touches>,
    mut first_ray_hit: Local<Option<Vec3>>,
    mut camera_writer: EventWriter<ControlEvent>,
    mut saved_smoother_weight: Local<f32>,
    mut over_threshold: Local<bool>,
    mut first_screen_touch: Local<Option<Vec2>>,
    ray_map: Res<RayMap>,
) {
    let Ok((camera_entity, look_transform, mut smoother, controller)) = cam_q.get_single_mut()
    else {
        return;
    };

    let intersection = get_plane_intersection_point(controller, ray_map.map(), camera_entity).map(
        |(pointer_id, point)| {
            (
                pointer_id
                    .get_touch_id()
                    .and_then(|id| touches.get_pressed(id).map(|touch| touch.position())),
                point,
            )
        },
    );

    if let Err(TouchIntersectionPointError::NoIntersection) = intersection {
        warn!("Touch Grab pan intersection did not intersect with Grab plane");
    }

    if first_ray_hit.is_none() {
        match intersection {
            Ok((Some(touch_pos), intersection_point)) => {
                info!("Touch grab pan started");

                *saved_smoother_weight = smoother.lag_weight;
                smoother.lag_weight = 0.1;

                *first_ray_hit = Some(intersection_point);
                *over_threshold = false;

                *first_screen_touch = Some(touch_pos);
            }
            Ok((None, _)) => {
                warn!("Tried to start Touch grab pan but no touch position was found");
            }
            _ => {}
        }
    }

    if (intersection == Err(TouchIntersectionPointError::NoTouchRay)
        || intersection == Err(TouchIntersectionPointError::MultipleTouchRays))
        && first_ray_hit.is_some()
    {
        info!("Touch grab pan stopped");
        smoother.lag_weight = *saved_smoother_weight;
        *first_ray_hit = None;
        inputs.clear_last_touches();
    }

    if let (Ok((Some(touch_pos), point)), Some(first_hit), Some(screen_touch)) =
        (intersection, *first_ray_hit, *first_screen_touch)
    {
        // Compensate for look transform smoothing to prevent jittering
        let smoothing_target_diff = if let Some(smoothing_transform) = smoother.lerp_tfm {
            look_transform.target - smoothing_transform.target
        } else {
            Vec3::ZERO
        };

        let first_hit_diff = first_hit - point - smoothing_target_diff;

        if touch_pos.distance(screen_touch) > 3.0 || *over_threshold {
            *over_threshold = true;
            camera_writer.send(ControlEvent::TranslateTarget(first_hit_diff));
        }
    }
}

fn get_plane_intersection_point(
    controller: &CameraController,
    ray_map: &HashMap<RayId, Ray3d>,
    camera_entity: Entity,
) -> Result<(PointerId, Vec3), TouchIntersectionPointError> {
    let mut filtered_map = ray_map
        .iter()
        .filter(|(ray_id, _)| ray_id.pointer.is_touch() && ray_id.camera == camera_entity)
        .map(|(ray_id, ray)| (ray_id.pointer, ray))
        .collect::<Vec<_>>();

    if filtered_map.len() > 1 {
        // Multiple touches
        return Err(TouchIntersectionPointError::MultipleTouchRays);
    }

    let (pointer_id, ray) = filtered_map
        .pop()
        .ok_or(TouchIntersectionPointError::NoTouchRay)?;

    ray.intersect_plane(
        Vec3::Y * controller.grab_height,
        InfinitePlane3d { normal: Dir3::Y },
    )
    .map(|distance: f32| (pointer_id, ray.get_point(distance)))
    .ok_or(TouchIntersectionPointError::NoIntersection)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TouchIntersectionPointError {
    NoIntersection,
    MultipleTouchRays,
    NoTouchRay,
}
