use bevy::prelude::*;

use crate::{CameraPerspectiveState, LookTransform};

/// Change between Perspective and Orthographic projection
///
/// Orthographic projection will move the camera to look down
pub(crate) struct ChangeProjectionPlugin;

impl Plugin for ChangeProjectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SavedCamera>();

        app.add_systems(
            OnEnter(CameraPerspectiveState::Perspective),
            enter_perspective,
        );
        app.add_systems(
            OnEnter(CameraPerspectiveState::Orthographic),
            enter_orthographic,
        );
    }
}

fn enter_perspective(
    mut query: Query<(&mut LookTransform, &mut Projection)>,
    mut saved_proj: ResMut<SavedCamera>,
) {
    if let Ok((mut transform, mut projection)) = query.get_single_mut() {
        if let Projection::Perspective(_persp) = projection.as_ref() {
            return;
        }
        let (new_projection, saved_transform) = if let Some(saved) = saved_proj.take() {
            saved
        } else {
            (PerspectiveProjection::default(), transform.clone())
        };
        bevy::log::info!(
            "Setting camera to perspective: {:?}, {:?}",
            new_projection,
            saved_transform
        );

        *projection = Projection::Perspective(new_projection);
        transform.set_if_neq(saved_transform);
    }
}

fn enter_orthographic(
    mut query: Query<(&mut LookTransform, &mut Projection)>,
    mut saved_proj: ResMut<SavedCamera>,
) {
    if let Ok((mut transform, mut projection)) = query.get_single_mut() {
        // Only change if the current projection on the camera is perspective
        let Projection::Perspective(persp) = projection.as_ref() else {
            log::error!("Camera was not in perspective mode! doing nothing");
            return;
        };
        saved_proj.0 = Some((persp.clone(), transform.clone()));

        let new_projection = ortho_from_looktransform(&mut transform);

        *projection = new_projection
    }
}

fn ortho_from_looktransform(transform: &mut LookTransform) -> Projection {
    if let Some(dir) = transform.look_direction() {
        if dir != Vec3::Y {
            transform.eye = transform.target + transform.radius() * Vec3::Y;
            // Use dir Vec3 as base for a normalized "up" clear Y axis
            let mut normalized_up = dir.normalize();
            normalized_up.y = 0.0;
            // Normalize again after y is changed
            transform.up = normalized_up.normalize();
        }
    }

    let new_projection = OrthographicProjection {
        scaling_mode: bevy::render::camera::ScalingMode::FixedVertical(
            transform.eye.y.abs() / 18.0,
        ),
        far: 5000.0,
        scale: 15.0,
        ..default()
    }
    .into();
    new_projection
}

#[derive(Resource, Deref, DerefMut, Default)]
struct SavedCamera(Option<(PerspectiveProjection, LookTransform)>);
