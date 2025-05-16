use bevy::ecs::system::SystemParam;
use bevy::input::touch::Touch;
use bevy::prelude::*;

#[derive(Resource, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct TouchInputSettings {
    allowed_pinch_delta_diff: f32,
    pinch_threshold: f32,
    drag_threshold: f32,
}

impl Default for TouchInputSettings {
    fn default() -> Self {
        Self {
            pinch_threshold: 1.0,
            allowed_pinch_delta_diff: 1.0,
            drag_threshold: 5.0,
        }
    }
}

pub(super) struct Pinch {
    /// The delta of the distances between the current touches compared to last frame.
    pub distance_delta: f32,
    /// The screen location between the current touches
    pub middle: Vec2,
}

/// Only one of [`Self::get_pinch`], [`Self::get_one_touch_drag`] and [`Self::get_two_touch_drag`]
/// can be used in the same system, otherwise they will overwrite each-others last touches.
#[derive(SystemParam)]
pub(super) struct TouchInputs<'w, 's> {
    touch_settings: Res<'w, TouchInputSettings>,
    touches: Res<'w, Touches>,
    last_touch_1: Local<'s, Option<Vec2>>,
    last_touch_2: Local<'s, Option<Vec2>>,
}

impl<'w, 's> TouchInputs<'w, 's> {
    fn get_two_touches(&mut self) -> Option<[(Touch, Vec2); 2]> {
        if self.touches.any_just_released() {
            *self.last_touch_1 = None;
            *self.last_touch_2 = None;
        }

        let touches: Vec<&Touch> = self.touches.iter().collect();

        // If less than or more than two touches, return None
        if touches.len() != 2 {
            return None;
        }

        let touch1 = touches[0].clone();
        let touch2 = touches[1].clone();

        let last1 = self.last_touch_1.unwrap_or(touch1.position());
        let last2 = self.last_touch_2.unwrap_or(touch2.position());

        *self.last_touch_1 = Some(touch1.position());
        *self.last_touch_2 = Some(touch2.position());

        Some([(touch1, last1), (touch2, last2)])
    }

    pub fn get_pinch(&mut self) -> Option<Pinch> {
        let [(touch1, last_a), (touch2, last_b)] = self.get_two_touches()?;

        let d1 = touch1.position() - last_a;
        let d2 = touch2.position() - last_b;

        if d1 == Vec2::ZERO || d2 == Vec2::ZERO {
            return None;
        }

        let distance = touch1.position().distance(touch2.position());

        if d1.dot(d2) > self.touch_settings.allowed_pinch_delta_diff {
            return None;
        }

        let previous_distance = touch1
            .previous_position()
            .distance(touch2.previous_position());

        let distance_delta = distance - previous_distance;

        //If the distance delta is below the threshold dont zoom
        if distance_delta.abs() < self.touch_settings.pinch_threshold {
            return None;
        }

        let middle = (touch1.previous_position() + touch2.previous_position()) / 2.0;

        Some(Pinch {
            distance_delta,
            middle,
        })
    }

    /// Can't be used in the same system as [`Self::get_pinch`]
    pub fn get_two_touch_drag(&mut self) -> Option<Vec2> {
        let [(touch1, last_a), (touch2, last_b)] = self.get_two_touches()?;

        let d1 = touch1.position() - last_a;
        let d2 = touch2.position() - last_b;

        let dist = d1.distance(d2);
        let avg = (d1 + d2) / 2.0;

        // If the touches arent moving together or the moved distance is too small, return Vec2::ZERO
        if dist > self.touch_settings.drag_threshold
            || avg.length_squared() < self.touch_settings.drag_threshold
        {
            return None;
        }

        Some(avg)
    }

    pub fn clear_last_touches(&mut self) {
        *self.last_touch_1 = None;
        *self.last_touch_2 = None;
    }
}
