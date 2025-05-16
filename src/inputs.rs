use std::marker::PhantomData;

use bevy::prelude::Reflect;
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
