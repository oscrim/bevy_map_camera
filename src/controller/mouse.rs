use bevy_app::prelude::*;
use bevy_camera::Camera;
use bevy_ecs::prelude::*;
use bevy_log::warn;
use bevy_math::{Dir3, Ray3d, Vec3, primitives::InfinitePlane3d};
use bevy_picking::{
    backend::ray::{RayId, RayMap},
    pointer::PointerId,
};
use bevy_platform::collections::HashMap;
use bevy_transform::components::GlobalTransform;
use bevy_window::{CursorIcon, PrimaryWindow, SystemCursorIcon, Window};

use super::ray_from_screenspace;

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
                .map(|alt| mouse_inputs.mouse_drag(alt))
                .flatten()
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

    let Some(target_distance) = ray.intersect_plane(
        Vec3::Y * controller.grab_height,
        InfinitePlane3d { normal: Dir3::Y },
    ) else {
        return;
    };

    let target = ray.get_point(target_distance);

    camera_writer.write(ControlMessage::Zoom {
        zoom_scalar: scalar,
        zoom_target: target,
    });
}

fn grab_pan(
    mut commands: Commands,
    cam_q: Single<(Entity, &CameraController), With<Camera>>,
    settings: Res<CameraControllerSettings>,
    inputs: Inputs,
    mut first_ray_hit: Local<Option<Vec3>>,
    primary_window_q: Single<Entity, With<PrimaryWindow>>,
    mut camera_writer: MessageWriter<ControlMessage>,
    ray_map: Res<RayMap>,
) {
    let (camera_entity, controller) = cam_q.into_inner();
    let window_entity = primary_window_q.into_inner();
    let drag_buttons = &settings.buttons.pan;

    if inputs.multi_pressed(drag_buttons) {
        let Some(intersection_point) =
            get_plane_intersection_point(controller, &ray_map.map, camera_entity)
        else {
            //Grab pan pressed without first ray hit, return
            return;
        };

        if let Some(first_hit) = *first_ray_hit {
            let first_hit_diff = first_hit - intersection_point;

            camera_writer.write(ControlMessage::TranslateTarget(first_hit_diff));
        } else {
            if let Ok(mut ecmd) = commands.get_entity(window_entity) {
                ecmd.entry::<CursorIcon>()
                    .and_modify(|mut icon| {
                        icon.set_if_neq(CursorIcon::System(SystemCursorIcon::Grabbing));
                    })
                    .or_insert(CursorIcon::System(SystemCursorIcon::Grabbing));
            }

            *first_ray_hit = Some(intersection_point);
        }
    } else if first_ray_hit.is_some() {
        *first_ray_hit = None;
        if let Ok(mut ecmd) = commands.get_entity(window_entity) {
            ecmd.entry::<CursorIcon>().and_modify(|mut icon| {
                icon.set_if_neq(CursorIcon::System(SystemCursorIcon::Default));
            });
        }
    }
}

fn get_plane_intersection_point(
    controller: &CameraController,
    ray_map: &HashMap<RayId, Ray3d>,
    camera_entity: Entity,
) -> Option<Vec3> {
    let ray_id = RayId {
        camera: camera_entity,
        pointer: PointerId::Mouse,
    };

    let Some(ray) = ray_map.get(&ray_id) else {
        warn!("No Ray3d for mouse pointer!");
        return None;
    };

    let intersection_point = ray
        .intersect_plane(
            Vec3::Y * controller.grab_height,
            InfinitePlane3d { normal: Dir3::Y },
        )
        .map(|distance: f32| ray.get_point(distance));

    if intersection_point.is_none() {
        warn!("Mouse Grab pan intersection did not intersect with Grab plane");
    };

    intersection_point
}
