use context::GraphicsContext;
use viewport::ViewportDimensions;
use window::{Window, WindowEvents};

pub mod window;
pub mod context;
pub mod vertex;
pub mod viewport;

fn main() {
    let window = Window::new();
    let mut context = pollster::block_on(GraphicsContext::new(&window));

    window.run(move |window, event| match event {
        WindowEvents::Resized { width, height } => {
            context.resize(width, height);
        },
        WindowEvents::Draw => {
            context.render().unwrap();
        }
        WindowEvents::Input { event } => {

        }
        _ => {}
    });
}
