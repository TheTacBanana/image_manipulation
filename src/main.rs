use context::GraphicsContext;
use input::MouseInput;
use texture::{load_bytes, Texture};
use window::{Window, WindowEvents};
use winit::{
    event::{DeviceEvent, Event, MouseButton, WindowEvent},
    event_loop::ControlFlow,
};

pub mod context;
pub mod texture;
pub mod vertex;
pub mod viewport;
pub mod window;
pub mod input;

fn main() {
    let window = Window::new();
    let mut context = pollster::block_on(GraphicsContext::new(&window));

    let args = std::env::args().collect::<Vec<_>>();
    let image_path = args.get(1).expect("Please specify image path");

    let texture = pollster::block_on(load_bytes(&image_path))
        .and_then(|bytes| Texture::from_bytes(&context, &bytes, ""))
        .expect("Failed to load image");

    window.run(move |window, event, control_flow| {
        context.egui.platform.handle_event(&event);
        match event {
            Event::RedrawRequested(_) => {
                context.render(&texture, &window).unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                context.resize(size.width, size.height);
            }
            Event::WindowEvent {
                window_id,
                event : WindowEvent::CursorMoved { position, .. },
            } => {
                context.process_input(MouseInput::Position(
                    cgmath::Vector2 { x: position.x as f32, y: position.y as f32 }
                ))
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button : MouseButton::Left, .. },
                ..
            } => {
                match state {
                    winit::event::ElementState::Pressed => context.process_input(MouseInput::ButtonPressed),
                    winit::event::ElementState::Released => context.process_input(MouseInput::ButtonReleased),
                }
            }
            Event::LoopDestroyed
            | Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared | Event::UserEvent(_) => {
                window.request_redraw();
            }
            _ => (),
        }
    });
}
