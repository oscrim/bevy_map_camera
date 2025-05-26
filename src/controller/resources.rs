use crate::inputs::InputButton;
use bevy_input::{keyboard::KeyCode, mouse::MouseButton};
use bevy_reflect::Reflect;

#[derive(Clone, Reflect)]
pub struct CameraControllerButtons {
    pub pan: Vec<InputButton>,
    /// Alternative pan key-binding
    pub pan_alt: Option<Vec<InputButton>>,
    pub rotate: Vec<InputButton>,
    /// Alternative rotate key-binding
    pub rotate_alt: Option<Vec<InputButton>>,
}
impl Default for CameraControllerButtons {
    fn default() -> Self {
        Self {
            pan: vec![MouseButton::Left.into()],
            rotate: vec![MouseButton::Left.into(), KeyCode::ShiftLeft.into()],
            pan_alt: None,
            rotate_alt: Some(vec![MouseButton::Right.into()]),
        }
    }
}
