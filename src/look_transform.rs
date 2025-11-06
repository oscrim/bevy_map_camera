use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_reflect::prelude::*;
use bevy_transform::components::Transform;

/// An eye and the target it's looking at. As a component, this can be modified in place of bevy's `Transform`, and the two will
/// stay in sync.
#[derive(Component, Debug, PartialEq, Clone, Copy, Reflect)]
#[reflect(Component, Default, Debug, PartialEq)]
#[require(Transform = default_transform())]
pub struct LookTransform {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
}

fn default_transform() -> Transform {
    let look_transform = LookTransform::default();

    Transform::from_translation(look_transform.eye).looking_at(look_transform.target, Vec3::Y)
}

impl From<LookTransform> for Transform {
    fn from(t: LookTransform) -> Self {
        eye_look_at_target_transform(t.eye, t.target, t.up)
    }
}

impl Default for LookTransform {
    fn default() -> Self {
        Self {
            eye: Vec3::ONE * 5.0,
            target: Vec3::ZERO,
            up: Vec3::Y,
        }
    }
}

impl LookTransform {
    pub fn new(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        Self { eye, target, up }
    }

    pub fn radius(&self) -> f32 {
        (self.target - self.eye).length()
    }

    pub fn look_direction(&self) -> Option<Vec3> {
        (self.target - self.eye).try_normalize()
    }
}

fn eye_look_at_target_transform(eye: Vec3, target: Vec3, up: Vec3) -> Transform {
    // If eye and target are very close, we avoid imprecision issues by keeping the look vector a unit vector.
    let look_vector = (target - eye).normalize();
    let look_at = eye + look_vector;

    Transform::from_translation(eye).looking_at(look_at, up)
}

#[cfg(feature = "bevy_easings")]
impl bevy_easings::Lerp for LookTransform {
    type Scalar = f32;
    fn lerp(&self, other: &Self, scalar: &Self::Scalar) -> Self {
        Self {
            eye: self.eye.lerp(other.eye, *scalar),
            target: self.target.lerp(other.target, *scalar),
            up: self.up.lerp(other.up, *scalar),
        }
    }
}

#[cfg(feature = "bevy_tweening")]
pub struct LookTransformLens {
    pub start: LookTransform,
    pub end: LookTransform,
}

#[cfg(feature = "bevy_tweening")]
impl bevy_tweening::Lens<LookTransform> for LookTransformLens {
    fn lerp(&mut self, mut target: Mut<LookTransform>, ratio: f32) {
        target.eye = self.start.eye.lerp(self.end.eye, ratio);
        target.target = self.start.target.lerp(self.end.target, ratio);
        target.up = self.start.up.lerp(self.end.up, ratio);
    }
}
