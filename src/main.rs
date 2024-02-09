use context::GraphicsContext;
use texture::{load_bytes, Texture};
use window::{Window, WindowEvents};
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

pub mod context;
pub mod texture;
pub mod vertex;
pub mod viewport;
pub mod window;

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
            Event::LoopDestroyed | Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared | Event::UserEvent(_) => {
                window.request_redraw();
            }
            _ => ()
        }
    });
}
