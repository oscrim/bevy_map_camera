use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_log::{info, warn};
use bevy_math::{Dir3, Ray3d, Vec3, primitives::InfinitePlane3d};
use bevy_picking::{
    backend::ray::{RayId, RayMap},
    pointer::PointerId,
};
use bevy_platform::collections::HashMap;
use bevy_render::camera::Camera;
use bevy_transform::components::GlobalTransform;
use bevy_window::{PrimaryWindow, SystemCursorIcon, Window};
use bevy_winit::cursor::CursorIcon;

use super::ray_from_screenspace;
use crate::look_transform::Smoother;

use super::{
    CameraController, CameraControllerSettings, ControlEvent, mouse_input::MouseKeyboardInputs,
};
use crate::{CameraChange, LookTransform, inputs::Inputs};

pub(super) struct MouseController;

impl Plugin for MouseController {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (zoom_orbit_camera, rotate_orbit_camera, grab_pan)
                .chain()
                .in_set(CameraChange::Before),
        );
    }
}

/// Handles the rotation of the camera
fn rotate_orbit_camera(
    settings: Res<CameraControllerSettings>,
    mut camera_writer: EventWriter<ControlEvent>,
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

    camera_writer.write(ControlEvent::Orbit(
        rotation_move * settings.mouse_rotation_sensitivity_modifier,
    ));
}

/// Handles the zooming of the orbital camera
fn zoom_orbit_camera(
    cam_q: Single<(&CameraController, &Camera, &GlobalTransform, &LookTransform)>,
    settings: Res<CameraControllerSettings>,
    main_window: Single<&Window, With<PrimaryWindow>>,
    mut mouse_inputs: MouseKeyboardInputs,
    mut camera_writer: EventWriter<ControlEvent>,
) {
    let (controller, camera, camera_gt, camera_lt) = cam_q.into_inner();
    let window = main_window.into_inner();

    let scroll_sensitivity = settings.mouse_zoom_sensitivity_modifier;

    let Some(scalar) = mouse_inputs.scroll_scalar(controller.pixels_per_line, scroll_sensitivity)
    else {
        return;
    };

    let Some(mouse_pos) = window.cursor_position() else {
        camera_writer.write(ControlEvent::Zoom {
            zoom_scalar: scalar,
            zoom_target: camera_lt.target,
        });

        return;
    };

    let Ok(ray) = ray_from_screenspace(mouse_pos, camera, camera_gt, window) else {
        camera_writer.write(ControlEvent::Zoom {
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

    camera_writer.write(ControlEvent::Zoom {
        zoom_scalar: scalar,
        zoom_target: target,
    });
}

fn grab_pan(
    mut commands: Commands,
    cam_q: Single<(Entity, &LookTransform, &mut Smoother, &CameraController), With<Camera>>,
    settings: Res<CameraControllerSettings>,
    inputs: Inputs,
    mut first_ray_hit: Local<Option<Vec3>>,
    primary_window_q: Single<Entity, With<PrimaryWindow>>,
    mut camera_writer: EventWriter<ControlEvent>,
    mut saved_smoother_weight: Local<f32>,
    ray_map: Res<RayMap>,
) {
    let (camera_entity, look_transform, mut smoother, controller) = cam_q.into_inner();
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
            // Compensate for look transform smoothing to prevent jittering
            let smoothing_target_diff = if let Some(smoothing_transform) = smoother.lerp_tfm {
                look_transform.target - smoothing_transform.target
            } else {
                Vec3::ZERO
            };

            let first_hit_diff = first_hit - intersection_point - smoothing_target_diff;

            camera_writer.write(ControlEvent::TranslateTarget(first_hit_diff));
        } else {
            info!("Mouse grab pan started");

            if let Ok(mut ecmd) = commands.get_entity(window_entity) {
                ecmd.entry::<CursorIcon>()
                    .and_modify(|mut icon| {
                        icon.set_if_neq(CursorIcon::System(SystemCursorIcon::Grabbing));
                    })
                    .or_insert(CursorIcon::System(SystemCursorIcon::Grabbing));
            }

            *saved_smoother_weight = smoother.lag_weight;
            smoother.lag_weight = 0.1;

            *first_ray_hit = Some(intersection_point);
        }
    } else if first_ray_hit.is_some() {
        info!("Mouse grab pan stopped");
        smoother.lag_weight = *saved_smoother_weight;
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
