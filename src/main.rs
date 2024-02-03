use context::GraphicsContext;
use texture::{load_bytes, Texture};
use window::{Window, WindowEvents};

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
        .and_then(|bytes| Texture::from_bytes(&context, &bytes, "")).expect("Failed to load image");

    window.run(move |window, event| match event {
        WindowEvents::Resized { width, height } => {
            context.resize(width, height);
        }
        WindowEvents::Draw => {
            context.render(&texture).unwrap();
        }
        _ => {}
    });
}
