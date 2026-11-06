use bevy_camera::primitives::Aabb;
use bevy_ecs::{prelude::*, system::SystemParam};
use bevy_math::{Dir3, Ray3d, Vec3, primitives::InfinitePlane3d};
use bevy_mesh::Mesh3d;
use bevy_picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings, RayCastVisibility};
use bevy_reflect::prelude::*;

use super::CameraController;
use crate::{LookTransform, look_angles::LookAngles};

/// Marks a meshed entity as a grab surface.
///
/// While at least one grab surface exists, the camera pans and zooms against these meshes
/// instead of the invisible plane at [`CameraController::grab_height`], and the camera target
/// follows the surface in the centre of the view.
///
/// Add it to the same entity that holds the `Mesh3d`; a marker on a parent entity is ignored.
/// The mesh also needs an `Aabb`, so entities with `NoFrustumCulling` or `NoAutoAabb` (without a
/// manual `Aabb`) and meshes that are still loading don't count yet. Hidden meshes are still used,
/// so an invisible collision mesh works too. If no grab surface exists, the camera behaves as before.
#[derive(Component, Reflect, Default, Clone, Copy, Debug)]
#[reflect(Component, Default, Debug)]
pub struct GrabSurface;

/// The plane a pan drag moves along, decided when the drag starts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DragPlane {
    /// The horizontal plane at [`CameraController::grab_height`], re-read every frame.
    GrabHeight,
    /// The horizontal plane through the point where a [`GrabSurface`] was grabbed.
    Surface { origin: Vec3 },
    /// The camera-facing plane through the target, used when the grab missed every [`GrabSurface`].
    View { origin: Vec3, normal: Dir3 },
}

impl DragPlane {
    pub(crate) fn intersect(&self, ray: Ray3d, controller: &CameraController) -> Option<Vec3> {
        match *self {
            DragPlane::GrabHeight => intersect_grab_height(ray, controller),
            DragPlane::Surface { origin } => intersect(ray, origin, Dir3::Y),
            DragPlane::View { origin, normal } => intersect(ray, origin, normal),
        }
    }

    /// True if dragging along this plane should leave the target floating instead of
    /// following the grab surfaces.
    pub(crate) fn is_floating(&self) -> bool {
        matches!(self, DragPlane::View { .. })
    }
}

/// Whether the camera target follows the [`GrabSurface`] in the centre of the view.
///
/// The target floats after a pan that started outside every grab surface, and follows the
/// surfaces again after a pan that starts on one.
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub(crate) struct SurfaceFollow {
    pub(crate) floating: bool,
}

impl SurfaceFollow {
    /// The state after a pan along `plane` has started.
    pub(crate) fn after_drag_on(plane: &DragPlane) -> Self {
        Self {
            floating: plane.is_floating(),
        }
    }
}

fn intersect(ray: Ray3d, origin: Vec3, normal: Dir3) -> Option<Vec3> {
    ray.intersect_plane(origin, InfinitePlane3d { normal })
        .map(|distance| ray.get_point(distance))
}

/// Intersects `ray` with the horizontal plane at the controller's grab height.
pub(crate) fn intersect_grab_height(ray: Ray3d, controller: &CameraController) -> Option<Vec3> {
    intersect(ray, Vec3::Y * controller.grab_height, Dir3::Y)
}

/// A [`GrabSurface`] that mesh ray casts can actually hit.
pub(crate) type HittableSurface = (With<GrabSurface>, With<Mesh3d>, With<Aabb>);

/// Ray casts against [`GrabSurface`] meshes.
#[derive(SystemParam)]
pub(crate) struct GrabSurfaces<'w, 's> {
    // Optional so the controller keeps working in apps without mesh assets.
    ray_cast: Option<MeshRayCast<'w, 's>>,
    surfaces: Query<'w, 's, (), HittableSurface>,
}

impl GrabSurfaces<'_, '_> {
    /// True if any hittable [`GrabSurface`] exists and mesh ray casting is available.
    pub(crate) fn is_active(&self) -> bool {
        self.ray_cast.is_some() && !self.surfaces.is_empty()
    }

    /// The nearest point where `ray` hits a [`GrabSurface`] mesh.
    pub(crate) fn cast(&mut self, ray: Ray3d) -> Option<Vec3> {
        let surfaces = &self.surfaces;
        let ray_cast = self.ray_cast.as_mut()?;
        let filter = |entity| surfaces.contains(entity);
        let settings = MeshRayCastSettings::default()
            .with_filter(&filter)
            .with_visibility(RayCastVisibility::Any);

        ray_cast
            .cast_ray(ray, &settings)
            .first()
            .map(|(_, hit)| hit.point)
    }

    /// Picks the plane to drag along for a pan starting with `ray`, and the point it was grabbed at.
    pub(crate) fn drag_plane(
        &mut self,
        ray: Ray3d,
        controller: &CameraController,
        look: &LookTransform,
    ) -> Option<(DragPlane, Vec3)> {
        let plane = if !self.is_active() {
            DragPlane::GrabHeight
        } else if let Some(hit) = self.cast(ray) {
            DragPlane::Surface { origin: hit }
        } else {
            DragPlane::View {
                origin: look.target,
                normal: view_normal(look)?,
            }
        };

        plane.intersect(ray, controller).map(|point| (plane, point))
    }

    /// The point `ray` points at, picked the same way as the start of a pan.
    pub(crate) fn pick_point(
        &mut self,
        ray: Ray3d,
        controller: &CameraController,
        look: &LookTransform,
    ) -> Option<Vec3> {
        self.drag_plane(ray, controller, look)
            .map(|(_, point)| point)
    }
}

/// Normal of the plane through the target that faces the camera.
///
/// When the camera is pitched as far as it goes (almost top down) the plane is horizontal
/// instead, so panning only moves in the XZ-plane.
fn view_normal(look: &LookTransform) -> Option<Dir3> {
    let to_eye = look.eye - look.target;
    if LookAngles::from_vector(to_eye).is_at_max_pitch() {
        return Some(Dir3::Y);
    }
    Dir3::new(-to_eye).ok()
}
