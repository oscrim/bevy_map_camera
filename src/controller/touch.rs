use super::{
    CameraController, ray_from_screenspace,
    surface::{DragPlane, GrabSurfaces, SurfaceFollow},
};
use bevy_app::{App, Plugin, PreUpdate};
use bevy_camera::Camera;
use bevy_ecs::prelude::*;
use bevy_input::touch::Touches;
use bevy_log::{error, warn};
use bevy_math::{Ray3d, Vec2, Vec3};
use bevy_picking::{
    backend::ray::{RayId, RayMap},
    pointer::PointerId,
};
use bevy_platform::collections::HashMap;
use bevy_transform::components::GlobalTransform;
use bevy_window::{PrimaryWindow, Window};

use crate::{CameraChange, LookTransform};

use super::{
    CameraControllerSettings, ControlMessage,
    touch_inputs::{Pinch, TouchInputSettings, TouchInputs},
};

pub(super) struct TouchInputPlugin;

impl Plugin for TouchInputPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TouchInputSettings>();
        app.init_resource::<TouchInputSettings>();
        app.add_systems(
            PreUpdate,
            (zoom_orbit_camera, grab_pan, rotate_orbit_camera).in_set(CameraChange::Before),
        );
    }
}

/// Handles the zooming of the orbital camera
fn zoom_orbit_camera(
    mut touches: TouchInputs,
    cam_q: Single<(&Camera, &GlobalTransform, &LookTransform, &CameraController)>,
    main_window: Single<&Window, With<PrimaryWindow>>,
    settings: Res<CameraControllerSettings>,
    mut camera_writer: MessageWriter<ControlMessage>,
    mut surfaces: GrabSurfaces,
) {
    // Get the deltas of the two touches
    let Some(Pinch {
        distance_delta,
        middle,
    }) = touches.get_pinch()
    else {
        return;
    };

    let (camera, camera_gt, camera_lt, controller) = cam_q.into_inner();
    let window = main_window.into_inner();

    let scalar = 1.0 - distance_delta * settings.touch_zoom_sensitivity_modifier;

    if scalar == 1.0 {
        return;
    }

    let Ok(ray) = ray_from_screenspace(middle, camera, camera_gt, window) else {
        camera_writer.write(ControlMessage::Zoom {
            zoom_scalar: scalar,
            zoom_target: camera_lt.target,
        });
        error!("Unable to create ray from screenspace");
        return;
    };

    let Some(target) = surfaces.pick_point(ray, controller, camera_lt, settings.minimum_pan_angle)
    else {
        warn!("Cursor click did not intersect with Grab plane");
        return;
    };

    camera_writer.write(ControlMessage::Zoom {
        zoom_scalar: scalar,
        zoom_target: target,
    });
}

/// Handles the rotation of the orbital camera, dont run in orthographic
fn rotate_orbit_camera(
    mut touches: TouchInputs,
    settings: Res<CameraControllerSettings>,
    mut camera_writer: MessageWriter<ControlMessage>,
) {
    let Some(rotation_move) = touches.get_two_touch_drag() else {
        return;
    };
    camera_writer.write(ControlMessage::Orbit(
        rotation_move * settings.touch_rotation_sensitivity_modifier,
    ));
}

#[allow(clippy::too_many_arguments)]
fn grab_pan(
    cam_q: Single<
        (
            Entity,
            &CameraController,
            &LookTransform,
            &mut SurfaceFollow,
        ),
        With<Camera>,
    >,
    mut inputs: TouchInputs,
    touches: Res<Touches>,
    mut first_ray_hit: Local<Option<(DragPlane, Vec3)>>,
    mut camera_writer: MessageWriter<ControlMessage>,
    mut over_threshold: Local<bool>,
    mut first_screen_touch: Local<Option<Vec2>>,
    ray_map: Res<RayMap>,
    settings: Res<CameraControllerSettings>,
    mut surfaces: GrabSurfaces,
) {
    let (camera_entity, controller, look, mut follow) = cam_q.into_inner();

    if !controller.enabled {
        *first_ray_hit = None;
        *first_screen_touch = None;
        return;
    }

    // Intersect with the plane of the ongoing drag, or pick a new plane if no drag has started
    let intersection = get_touch_ray(&ray_map.map, camera_entity).and_then(|(pointer_id, ray)| {
        let (plane, point) = match *first_ray_hit {
            Some((plane, _)) => plane
                .intersect(ray, controller, settings.minimum_pan_angle)
                .map(|point| (plane, point)),
            None => surfaces.drag_plane(ray, controller, look, settings.minimum_pan_angle),
        }
        .ok_or(TouchIntersectionPointError::NoIntersection)?;

        let touch_pos = pointer_id
            .get_touch_id()
            .and_then(|id| touches.get_pressed(id).map(|touch| touch.position()));

        Ok((touch_pos, plane, point))
    });

    if let Err(TouchIntersectionPointError::NoIntersection) = intersection {
        warn!("Touch Grab pan intersection did not intersect with Grab plane");
    }

    if first_ray_hit.is_none() {
        match intersection {
            Ok((Some(touch_pos), plane, intersection_point)) => {
                *first_ray_hit = Some((plane, intersection_point));
                *over_threshold = false;

                *first_screen_touch = Some(touch_pos);
            }
            Ok((None, _, _)) => {
                warn!("Tried to start Touch grab pan but no touch position was found");
            }
            _ => {}
        }
    }

    if (intersection == Err(TouchIntersectionPointError::NoTouchRay)
        || intersection == Err(TouchIntersectionPointError::MultipleTouchRays))
        && first_ray_hit.is_some()
    {
        *first_ray_hit = None;
        inputs.clear_last_touches();
    }

    if let (Ok((Some(touch_pos), _, point)), Some((plane, first_hit)), Some(screen_touch)) =
        (intersection, *first_ray_hit, *first_screen_touch)
    {
        let first_hit_diff = first_hit - point;

        if (touch_pos.distance(screen_touch) > 3.0 || *over_threshold)
            && first_hit_diff.length_squared() > 1e-10
        {
            if !*over_threshold {
                // Only an actual pan changes whether the target follows the surfaces, not a tap
                follow.set_if_neq(SurfaceFollow::after_drag_on(&plane));
            }
            *over_threshold = true;
            camera_writer.write(ControlMessage::TranslateTarget(first_hit_diff));
        }
    }
}

fn get_touch_ray(
    ray_map: &HashMap<RayId, Ray3d>,
    camera_entity: Entity,
) -> Result<(PointerId, Ray3d), TouchIntersectionPointError> {
    let mut filtered_map = ray_map
        .iter()
        .filter(|(ray_id, _)| ray_id.pointer.is_touch() && ray_id.camera == camera_entity)
        .map(|(ray_id, ray)| (ray_id.pointer, *ray))
        .collect::<Vec<_>>();

    if filtered_map.len() > 1 {
        // Multiple touches
        return Err(TouchIntersectionPointError::MultipleTouchRays);
    }

    filtered_map
        .pop()
        .ok_or(TouchIntersectionPointError::NoTouchRay)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TouchIntersectionPointError {
    NoIntersection,
    MultipleTouchRays,
    NoTouchRay,
}
