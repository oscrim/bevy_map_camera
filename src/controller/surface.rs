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
    /// A camera-facing plane. Used through the target when the grab missed every [`GrabSurface`]
    /// (`floating`), and through the hit when a surface was grabbed at a grazing angle.
    View {
        origin: Vec3,
        normal: Dir3,
        floating: bool,
    },
}

impl DragPlane {
    /// Intersects `ray` with the plane. For horizontal planes the ray is first kept at least
    /// `min_angle` (radians) away from running along the plane, so the point stays within reach.
    pub(crate) fn intersect(
        &self,
        ray: Ray3d,
        controller: &CameraController,
        min_angle: f32,
    ) -> Option<Vec3> {
        match *self {
            DragPlane::GrabHeight => {
                bounded_plane_point(ray, Vec3::Y * controller.grab_height, Dir3::Y, min_angle)
            }
            DragPlane::Surface { origin } => bounded_plane_point(ray, origin, Dir3::Y, min_angle),
            DragPlane::View { origin, normal, .. } => {
                bounded_plane_point(ray, origin, normal, min_angle)
            }
        }
    }

    /// True if dragging along this plane should leave the target floating instead of
    /// following the grab surfaces.
    pub(crate) fn is_floating(&self) -> bool {
        matches!(self, DragPlane::View { floating: true, .. })
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

/// Intersects `ray` with the plane through `origin`. For a horizontal plane the ray is first
/// tilted towards the plane, keeping its compass direction, until it meets the plane at an angle
/// of at least `min_angle`.
///
/// Without this a ray close to the horizon hits the plane extremely far away, so a tiny cursor
/// move flings the camera. With it the reachable distance is at most `height / tan(min_angle)`,
/// and rays pointing away from the plane still give a point.
fn bounded_plane_point(ray: Ray3d, origin: Vec3, normal: Dir3, min_angle: f32) -> Option<Vec3> {
    if normal != Dir3::Y {
        return intersect(ray, origin, normal);
    }

    // Which way the ray must point to reach the plane: down (-1) or up (+1)
    let towards = (origin.y - ray.origin.y).signum();
    let (sin_min, cos_min) = min_angle.sin_cos();

    if towards * ray.direction.y >= sin_min {
        return intersect(ray, origin, normal);
    }

    let Some(horizontal) = Vec3::new(ray.direction.x, 0.0, ray.direction.z).try_normalize() else {
        return intersect(ray, origin, normal);
    };
    let direction = Dir3::new(horizontal * cos_min + Vec3::Y * towards * sin_min).ok()?;

    intersect(Ray3d::new(ray.origin, direction), origin, normal)
}

/// The plane to drag along after `ray` hit a [`GrabSurface`] at `hit`.
///
/// Normally that is the horizontal plane through the hit. If the ray meets that plane at less
/// than `min_angle`, it is nearly edge-on to the camera, so the camera-facing plane through the
/// hit is used instead. That keeps the grabbed point under the cursor.
fn surface_plane(hit: Vec3, ray: Ray3d, look: &LookTransform, min_angle: f32) -> Option<DragPlane> {
    if ray.direction.y.abs() >= min_angle.sin() {
        return Some(DragPlane::Surface { origin: hit });
    }

    Some(DragPlane::View {
        origin: hit,
        normal: Dir3::new(look.target - look.eye).ok()?,
        floating: false,
    })
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
    ///
    /// `min_angle` is [`CameraControllerSettings::minimum_pan_angle`](super::CameraControllerSettings::minimum_pan_angle).
    pub(crate) fn drag_plane(
        &mut self,
        ray: Ray3d,
        controller: &CameraController,
        look: &LookTransform,
        min_angle: f32,
    ) -> Option<(DragPlane, Vec3)> {
        let plane = if !self.is_active() {
            DragPlane::GrabHeight
        } else if let Some(hit) = self.cast(ray) {
            surface_plane(hit, ray, look, min_angle)?
        } else {
            DragPlane::View {
                origin: look.target,
                normal: view_normal(look)?,
                floating: true,
            }
        };

        plane
            .intersect(ray, controller, min_angle)
            .map(|point| (plane, point))
    }

    /// The point `ray` points at, picked the same way as the start of a pan.
    pub(crate) fn pick_point(
        &mut self,
        ray: Ray3d,
        controller: &CameraController,
        look: &LookTransform,
        min_angle: f32,
    ) -> Option<Vec3> {
        self.drag_plane(ray, controller, look, min_angle)
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

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;

    const MIN_ANGLE: f32 = 10.0 * std::f32::consts::PI / 180.0;

    fn ray(origin: Vec3, direction: Vec3) -> Ray3d {
        Ray3d::new(origin, Dir3::new(direction).unwrap())
    }

    #[test]
    fn steep_ray_is_unchanged() {
        let ray = ray(Vec3::new(0.0, 10.0, 0.0), Vec3::new(1.0, -1.0, 0.0));
        let point = bounded_plane_point(ray, Vec3::ZERO, Dir3::Y, MIN_ANGLE).unwrap();
        assert_relative_eq!(point.x, 10.0, epsilon = 1e-4);
        assert_relative_eq!(point.y, 0.0, epsilon = 1e-4);
    }

    #[test]
    fn near_horizon_ray_is_bounded() {
        let height = 10.0;
        let max_reach = height / MIN_ANGLE.tan();
        let ray = ray(Vec3::Y * height, Vec3::new(1.0, -0.001, 0.0));
        let point = bounded_plane_point(ray, Vec3::ZERO, Dir3::Y, MIN_ANGLE).unwrap();
        assert_relative_eq!(point.x, max_reach, epsilon = 1e-3);
        assert_relative_eq!(point.y, 0.0, epsilon = 1e-4);
    }

    #[test]
    fn ray_above_horizon_still_hits() {
        let height = 10.0;
        let ray = ray(Vec3::Y * height, Vec3::new(0.0, 0.5, -1.0));
        let point = bounded_plane_point(ray, Vec3::ZERO, Dir3::Y, MIN_ANGLE).unwrap();
        assert_relative_eq!(point.z, -height / MIN_ANGLE.tan(), epsilon = 1e-3);
    }

    #[test]
    fn camera_below_plane_is_bounded_too() {
        let ray = ray(Vec3::ZERO, Vec3::new(1.0, -0.2, 0.0));
        let point = bounded_plane_point(ray, Vec3::Y * 2.0, Dir3::Y, MIN_ANGLE).unwrap();
        assert_relative_eq!(point.y, 2.0, epsilon = 1e-4);
        assert_relative_eq!(point.x, 2.0 / MIN_ANGLE.tan(), epsilon = 1e-3);
    }

    #[test]
    fn grazing_surface_hit_uses_camera_facing_plane() {
        let look = LookTransform::new(Vec3::new(0.0, 5.0, 10.0), Vec3::ZERO, Vec3::Y);
        let hit = Vec3::new(0.0, 4.9, -10.0);
        let grazing = ray(look.eye, hit - look.eye);

        let plane = surface_plane(hit, grazing, &look, MIN_ANGLE).unwrap();
        assert!(matches!(
            plane,
            DragPlane::View {
                floating: false,
                ..
            }
        ));
        assert!(!plane.is_floating());

        let steep = ray(look.eye, Vec3::new(0.0, -1.0, -1.0));
        let plane = surface_plane(Vec3::new(0.0, 0.0, 5.0), steep, &look, MIN_ANGLE).unwrap();
        assert_eq!(
            plane,
            DragPlane::Surface {
                origin: Vec3::new(0.0, 0.0, 5.0)
            }
        );
    }
}
