use bevy_app::prelude::*;
use bevy_camera::Camera;
use bevy_ecs::prelude::*;
use bevy_log::warn;
use bevy_math::{Ray3d, Vec3};
use bevy_picking::{
    backend::ray::{RayId, RayMap},
    pointer::PointerId,
};
use bevy_platform::collections::HashMap;
use bevy_transform::components::GlobalTransform;
use bevy_window::{CursorIcon, PrimaryWindow, SystemCursorIcon, Window};

use super::{
    ray_from_screenspace,
    surface::{DragPlane, GrabSurfaces, SurfaceFollow},
};

use super::{
    CameraController, CameraControllerSettings, ControlMessage, mouse_input::MouseKeyboardInputs,
};
use crate::{CameraChange, LookTransform, inputs::Inputs};

pub(super) struct MouseController;

impl Plugin for MouseController {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (zoom_orbit_camera, rotate_orbit_camera, grab_pan)
                .chain()
                .in_set(CameraChange::Before),
        );
    }
}

/// Handles the rotation of the camera
fn rotate_orbit_camera(
    settings: Res<CameraControllerSettings>,
    mut camera_writer: MessageWriter<ControlMessage>,
    mut mouse_inputs: MouseKeyboardInputs,
) {
    let Some(rotation_move) = mouse_inputs
        .mouse_drag(&settings.buttons.rotate)
        .or_else(|| {
            settings
                .buttons
                .rotate_alt
                .as_ref()
                .and_then(|alt| mouse_inputs.mouse_drag(alt))
        })
    else {
        return;
    };

    camera_writer.write(ControlMessage::Orbit(
        rotation_move * settings.mouse_rotation_sensitivity_modifier,
    ));
}

/// Handles the zooming of the orbital camera
fn zoom_orbit_camera(
    cam_q: Single<(&CameraController, &Camera, &GlobalTransform, &LookTransform)>,
    settings: Res<CameraControllerSettings>,
    main_window: Single<&Window, With<PrimaryWindow>>,
    mut mouse_inputs: MouseKeyboardInputs,
    mut camera_writer: MessageWriter<ControlMessage>,
    mut surfaces: GrabSurfaces,
) {
    let (controller, camera, camera_gt, camera_lt) = cam_q.into_inner();
    let window = main_window.into_inner();

    let scroll_sensitivity = settings.mouse_zoom_sensitivity_modifier;

    let Some(scalar) = mouse_inputs.scroll_scalar(controller.pixels_per_line, scroll_sensitivity)
    else {
        return;
    };

    let Some(mouse_pos) = window.cursor_position() else {
        camera_writer.write(ControlMessage::Zoom {
            zoom_scalar: scalar,
            zoom_target: camera_lt.target,
        });

        return;
    };

    let Ok(ray) = ray_from_screenspace(mouse_pos, camera, camera_gt, window) else {
        camera_writer.write(ControlMessage::Zoom {
            zoom_scalar: scalar,
            zoom_target: camera_lt.target,
        });

        return;
    };

    let Some(target) = surfaces.zoom_point(ray, controller, camera_lt, settings.minimum_pan_angle)
    else {
        return;
    };

    camera_writer.write(ControlMessage::Zoom {
        zoom_scalar: scalar,
        zoom_target: target,
    });
}

#[allow(clippy::too_many_arguments)]
fn grab_pan(
    mut commands: Commands,
    cam_q: Single<
        (
            Entity,
            &CameraController,
            &LookTransform,
            &mut SurfaceFollow,
        ),
        With<Camera>,
    >,
    settings: Res<CameraControllerSettings>,
    inputs: Inputs,
    mut first_ray_hit: Local<Option<(DragPlane, Vec3)>>,
    mut moved: Local<bool>,
    primary_window_q: Single<Entity, With<PrimaryWindow>>,
    mut camera_writer: MessageWriter<ControlMessage>,
    ray_map: Res<RayMap>,
    mut surfaces: GrabSurfaces,
) {
    let (camera_entity, controller, look, mut follow) = cam_q.into_inner();
    let window_entity = primary_window_q.into_inner();
    let buttons = &settings.buttons;
    // A disabled controller behaves as if the pan buttons were released
    let pan_pressed = controller.enabled
        && (inputs.multi_pressed(&buttons.pan)
            || buttons
                .pan_alt
                .as_ref()
                .is_some_and(|alt| inputs.multi_pressed(alt)));

    if pan_pressed {
        let Some(ray) = get_mouse_ray(&ray_map.map, camera_entity) else {
            return;
        };

        if let Some((plane, first_hit)) = *first_ray_hit {
            let Some(intersection_point) =
                plane.intersect(ray, controller, settings.minimum_pan_angle)
            else {
                warn!("Mouse Grab pan intersection did not intersect with Grab plane");
                return;
            };

            let first_hit_diff = first_hit - intersection_point;
            if first_hit_diff.length_squared() <= 1e-10 {
                return;
            }

            if !*moved {
                // Only an actual pan changes whether the target follows the surfaces, not a click
                *moved = true;
                follow.set_if_neq(SurfaceFollow::after_drag_on(&plane));
            }

            camera_writer.write(ControlMessage::TranslateTarget(first_hit_diff));
        } else {
            let Some(drag_plane) =
                surfaces.drag_plane(ray, controller, look, settings.minimum_pan_angle)
            else {
                //Grab pan pressed without first ray hit, return
                warn!("Mouse Grab pan intersection did not intersect with Grab plane");
                return;
            };

            if let Ok(mut ecmd) = commands.get_entity(window_entity) {
                ecmd.entry::<CursorIcon>()
                    .and_modify(|mut icon| {
                        icon.set_if_neq(CursorIcon::System(SystemCursorIcon::Grabbing));
                    })
                    .or_insert(CursorIcon::System(SystemCursorIcon::Grabbing));
            }

            *first_ray_hit = Some(drag_plane);
            *moved = false;
        }
    } else if first_ray_hit.is_some() {
        *first_ray_hit = None;
        *moved = false;
        if let Ok(mut ecmd) = commands.get_entity(window_entity) {
            ecmd.entry::<CursorIcon>().and_modify(|mut icon| {
                icon.set_if_neq(CursorIcon::System(SystemCursorIcon::Default));
            });
        }
    }
}

fn get_mouse_ray(ray_map: &HashMap<RayId, Ray3d>, camera_entity: Entity) -> Option<Ray3d> {
    let ray_id = RayId {
        camera: camera_entity,
        pointer: PointerId::Mouse,
    };

    let ray = ray_map.get(&ray_id).copied();
    if ray.is_none() {
        warn!("No Ray3d for mouse pointer!");
    }
    ray
}
