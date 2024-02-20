use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
};

// Wrapper around an eventloop and a window
pub struct Window {
    pub event_loop: EventLoop<()>,
    pub raw: winit::window::Window,
}

impl Default for Window {
    fn default() -> Self {
        Self::new()
    }
}

impl Window {
    // Create a new Window
    pub fn new() -> Self {
        let event_loop = EventLoop::new();

        let raw = winit::window::Window::new(&event_loop).expect("Failed to create Window");

        #[cfg(target_arch = "wasm32")]
        {
            use winit::{dpi::PhysicalSize, platform::web::WindowExtWebSys};

            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("main-body")?;
                    raw.set_inner_size(PhysicalSize::new(1000 as f32, 1000 as f32));
                    let canvas = web_sys::Element::from(raw.canvas());
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        Self { event_loop, raw }
    }

    // Run the event loop with a callback
    pub fn run(
        self,
        mut callback: impl 'static + FnMut(&winit::window::Window, Event<'_, ()>, &mut ControlFlow),
    ) {
        self.event_loop.run(move |event, _, control_flow| {
            callback(&self.raw, event, control_flow);
        });
    }
}
