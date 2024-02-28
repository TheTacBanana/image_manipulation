use cgmath::{Vector2, Zero};

/// Enum to represent all cursor actions
#[derive(Debug, Clone, Copy)]
pub enum CursorEvent {
    ButtonPressed,
    ButtonReleased,
    Position(Vector2<f32>),
    Scroll(f32),
}

/// Stores data about the current input state
#[derive(Debug)]
pub struct InputContext {
    pub last_mouse_pos: Vector2<f32>,
    pub mouse_pressed: bool,
    pub mouse_over_ui: bool,
}

impl Default for InputContext {
    fn default() -> Self {
        Self {
            last_mouse_pos: Vector2::zero(),
            mouse_pressed: false,
            mouse_over_ui: false,
        }
    }
}
