use cgmath::Vector2;

#[derive(Debug, Clone, Copy)]
pub enum MouseInput {
    ButtonPressed,
    ButtonReleased,
    Position(Vector2<f32>)
}