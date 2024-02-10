use cgmath::Vector2;

#[derive(Debug, Clone, Copy)]
pub enum MouseInput {
    ButtonPressed,
    ButtonReleased,
    StartTouch(Vector2<f32>),
    EndTouch,
    Position(Vector2<f32>),
    Scroll(f32),
}