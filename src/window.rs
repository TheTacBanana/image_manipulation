use winit::{
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    dpi::PhysicalSize
};

#[derive(Debug)]
pub enum WindowEvents {
    Resized { width: u32, height: u32 },
    Input { event: InputEvent },
    Draw,
    Exit,
}

#[derive(Debug)]
pub enum InputEvent {
    Keyboard {
        state: ElementState,
        virtual_keycode: Option<VirtualKeyCode>,
    },
    MouseClick {
        state: ElementState,
        button: MouseButton,
    },
    MouseDelta {
        delta: cgmath::Vector2<f32>,
    },
}

pub struct Window {
    pub event_loop: EventLoop<()>,
    pub raw: winit::window::Window,
}

impl Window {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let raw = winit::window::Window::new(&event_loop).expect("Failed to create Window");

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;

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

    pub fn run(
        self,
        mut callback: impl 'static + FnMut(&winit::window::Window, Event<'_, ()>, &mut ControlFlow) -> (),
    ) {
        self.event_loop.run(move |event, _, mut control_flow| {
            callback(&self.raw, event, &mut control_flow);
        });
    }
}
