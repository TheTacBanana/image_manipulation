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
        mut callback: impl 'static + FnMut(&winit::window::Window, WindowEvents) -> (),
    ) {
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    callback(
                        &self.raw,
                        WindowEvents::Resized {
                            width: size.width,
                            height: size.height,
                        },
                    );
                    ControlFlow::Poll
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state,
                                    virtual_keycode,
                                    ..
                                },
                            ..
                        },
                    ..
                } => {
                    callback(
                        &self.raw,
                        WindowEvents::Input {
                            event: InputEvent::Keyboard {
                                state,
                                virtual_keycode,
                            },
                        },
                    );
                    ControlFlow::Poll
                }
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { state, button, .. },
                    ..
                } => {
                    callback(
                        &self.raw,
                        WindowEvents::Input {
                            event: InputEvent::MouseClick { state, button },
                        },
                    );
                    ControlFlow::Poll
                }
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    callback(
                        &self.raw,
                        WindowEvents::Input {
                            event: InputEvent::MouseDelta {
                                delta: cgmath::Vector2 {
                                    x: delta.0 as f32,
                                    y: delta.1 as f32,
                                },
                            },
                        },
                    );
                    ControlFlow::Poll
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    callback(&self.raw, WindowEvents::Exit);
                    ControlFlow::Exit
                }
                Event::RedrawRequested(_) => {
                    callback(&self.raw, WindowEvents::Draw);
                    ControlFlow::Poll
                }
                Event::LoopDestroyed => {
                    callback(&self.raw, WindowEvents::Exit);
                    ControlFlow::Exit
                }
                Event::MainEventsCleared => {
                    self.raw.request_redraw();
                    ControlFlow::Poll
                }
                _ => ControlFlow::Poll,
            }
        });
    }
}
