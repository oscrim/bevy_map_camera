use std::ops::Deref;

use crate::inputs::{InputButton, Inputs};
use bevy_ecs::{prelude::*, system::SystemParam};
use bevy_input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy_math::Vec2;

#[derive(SystemParam)]
pub(super) struct MouseKeyboardInputs<'w, 's> {
    inputs: Inputs<'w, 's>,
    ev_motion: EventReader<'w, 's, MouseMotion>,
    ev_scroll: EventReader<'w, 's, MouseWheel>,
}

impl<'w, 's> Deref for MouseKeyboardInputs<'w, 's> {
    type Target = Inputs<'w, 's>;
    fn deref(&self) -> &Self::Target {
        &self.inputs
    }
}

impl<'w, 's> MouseKeyboardInputs<'w, 's> {
    pub fn mouse_drag(&mut self, buttons: &Vec<InputButton>) -> Option<Vec2> {
        if !self.inputs.multi_pressed(buttons) {
            return None;
        }

        let sum = self.ev_motion.read().map(|e| e.delta).sum::<Vec2>();

        if sum.length_squared() > 0.0 {
            Some(sum)
        } else {
            None
        }
    }

    pub fn scroll_scalar(&mut self, pixels_per_line: f32, scroll_sensitivity: f32) -> Option<f32> {
        if self.ev_scroll.len() == 0 {
            return None;
        }

        let mut scalar = 1.0;
        for ev in self.ev_scroll.read() {
            let scroll_amount = match ev.unit {
                MouseScrollUnit::Line => ev.y,
                MouseScrollUnit::Pixel => ev.y / pixels_per_line,
            };

            scalar *= 1.0 - scroll_amount * scroll_sensitivity;
        }

        Some(scalar)
    }
}
