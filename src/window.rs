use winit::{
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
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
