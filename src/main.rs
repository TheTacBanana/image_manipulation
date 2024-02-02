use context::GraphicsContext;
use window::{Window, WindowEvents};

pub mod window;
pub mod context;
pub mod vertex;

fn main() {
    let window = Window::new();
    let mut context = pollster::block_on(GraphicsContext::new(&window));



    window.run(move |window, event| match event {
        WindowEvents::Resized { width, height } => context.resize(width, height),
        WindowEvents::Draw => {
            // let cur_cam = cams.get_mut(cur_cam).unwrap();
            // cur_cam.bind_camera(&context);

            // pipeline
            //     .draw(
            //         &context,
            //         &asset_storage.get_asset::<Model>(&model_handle),
            //         &cur_cam,
            //     )
            //     .expect("An error occured with the surface");

        }
        WindowEvents::Input { event } => {

        }
        _ => {}
    });
}
