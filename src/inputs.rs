use std::marker::PhantomData;

use bevy::prelude::Reflect;
use bevy::utils::hashbrown::HashSet;
use bevy::{
    ecs::system::SystemParam,
    prelude::{ButtonInput, KeyCode, MouseButton, Res},
};

#[derive(SystemParam)]
pub(crate) struct Inputs<'w, 's> {
    pub keys: Res<'w, ButtonInput<KeyCode>>,
    pub mouse: Res<'w, ButtonInput<MouseButton>>,
    #[system_param(ignore)]
    marker: PhantomData<&'s ()>,
}

impl<'w, 's> Inputs<'w, 's> {
    /// Returns true if only the buttons in `input` are pressed.
    pub fn multi_pressed(&self, input: &Vec<InputButton>) -> bool {
        if input.is_empty() {
            return false;
        }

        let currently_pressed = self.keys.get_pressed().count() + self.mouse.get_pressed().count();

        if currently_pressed != input.len() {
            return false;
        }

        let mut pressed = true;

        for button in input {
            pressed &= match button {
                InputButton::Mouse(mouse) => self.mouse.pressed(*mouse),
                InputButton::Key(key) => self.keys.pressed(*key),
            }
        }

        pressed
    }

    pub fn _just_pressed<T: Into<InputButton>>(&self, input: T) -> bool {
        let input = input.into();
        match input {
            InputButton::Mouse(mouse) => self.mouse.just_pressed(mouse),
            InputButton::Key(key) => self.keys.just_pressed(key),
        }
    }

    /// Returns true if the button combination was just pressed
    pub fn multi_just_pressed(&self, input: &Vec<InputButton>) -> bool {
        if input.is_empty() {
            return false;
        }

        let mut pressed_key = false;
        let key_ref = &mut pressed_key;
        let mut pressed_mouse = false;
        let key_mouse = &mut pressed_mouse;

        // Collect a Vec of all the pressed and just pressed inputs
        let mut pressed_and_just_pressed = self
            .keys
            .get_pressed()
            .map(|key| InputButton::from(*key))
            .chain(self.mouse.get_pressed().map(|key| InputButton::from(*key)))
            .chain(self.keys.get_just_pressed().map(|key| {
                *key_ref = true;
                InputButton::from(*key)
            }))
            .chain(self.mouse.get_just_pressed().map(|key| {
                *key_mouse = true;
                InputButton::from(*key)
            }))
            .collect::<HashSet<_>>();

        // No buttons were just pressed
        if !pressed_key && !pressed_mouse {
            return false;
        }

        for button in input {
            // If a button from the input is pressed or were not
            // released
            if !pressed_and_just_pressed.remove(button) {
                return false;
            }
        }

        // If all buttons in the input were pressed or released
        // the set should be empty
        pressed_and_just_pressed.is_empty()
    }

    /// Returns true if any of the buttons were just released
    pub fn multi_just_released(&self, input: &Vec<InputButton>) -> bool {
        if input.is_empty() {
            return false;
        }

        let mut released_key = false;
        let key_ref = &mut released_key;
        let mut released_mouse = false;
        let key_mouse = &mut released_mouse;

        // Collect a Vec of all the pressed and just released inputs
        let mut pressed_and_released = self
            .keys
            .get_pressed()
            .map(|key| InputButton::from(*key))
            .chain(self.mouse.get_pressed().map(|key| InputButton::from(*key)))
            .chain(self.keys.get_just_released().map(|key| {
                *key_ref = true;
                InputButton::from(*key)
            }))
            .chain(self.mouse.get_just_released().map(|key| {
                *key_mouse = true;
                InputButton::from(*key)
            }))
            .collect::<HashSet<_>>();

        // No buttons were released
        if !released_key && !released_mouse {
            return false;
        }

        for button in input {
            // If a button from the input is pressed or were not
            // released
            if !pressed_and_released.remove(button) {
                return false;
            }
        }

        // If all buttons in the input were pressed or released
        // the set should be empty
        pressed_and_released.is_empty()
    }
}

#[derive(Hash, Debug, Clone, Copy, Reflect, PartialEq, Eq)]
pub enum InputButton {
    Mouse(MouseButton),
    Key(KeyCode),
}
impl From<MouseButton> for InputButton {
    fn from(value: MouseButton) -> Self {
        Self::Mouse(value)
    }
}
impl From<KeyCode> for InputButton {
    fn from(value: KeyCode) -> Self {
        Self::Key(value)
    }
}
