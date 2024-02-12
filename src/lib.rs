use context::GraphicsContext;

use input::CursorEvent;
use pollster::FutureExt;
use texture::Texture;
use window::Window;
use winit::{
    event::{Event, MouseButton, MouseScrollDelta, Touch, TouchPhase, WindowEvent},
    event_loop::ControlFlow,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::texture::load_bytes;

pub mod context;
pub mod image_display;
pub mod input;
pub mod texture;
pub mod vertex;
pub mod window;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let window = Window::new();
    let mut context = pollster::block_on(GraphicsContext::new(&window));

    let mut texture = Texture::from_bytes(&context, include_bytes!("../assets/raytrace.jpg"), "").unwrap();

    window.run(move |window, event, control_flow| {
        if let Some(path) = context.egui.opened_file.take() {
            texture = Texture::from_bytes(
                &context,
                &load_bytes(path.clone().to_str().unwrap())
                    .block_on()
                    .unwrap(),
                "",
            )
            .unwrap();
        }

        context.egui.platform.handle_event(&event);
        match event {
            Event::RedrawRequested(_) => {
                context.render(&texture, window).unwrap();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                context.resize(size.width, size.height);
            }
            Event::WindowEvent {
                event: WindowEvent::DroppedFile(path),
                ..
            } => {
                texture = Texture::from_bytes(
                    &context,
                    &load_bytes(path.to_str().unwrap()).block_on().unwrap(),
                    "",
                )
                .unwrap();
            }
            Event::WindowEvent {
                event:
                    WindowEvent::Touch(Touch {
                        location: position,
                        phase,
                        id,
                        ..
                    }),
                ..
            } => {
                let pos = cgmath::Vector2 {
                    x: position.x as f32,
                    y: position.y as f32,
                };
                context.process_input(match phase {
                    TouchPhase::Started => CursorEvent::StartTouch(id, pos),
                    TouchPhase::Moved => CursorEvent::TouchMove(id, pos),
                    TouchPhase::Ended => CursorEvent::EndTouch(id),
                    TouchPhase::Cancelled => return,
                });
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => context.process_input(CursorEvent::Position(cgmath::Vector2 {
                x: position.x as f32,
                y: position.y as f32,
            })),
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => context.process_input(match delta {
                MouseScrollDelta::LineDelta(_, y) => CursorEvent::Scroll(y),
                MouseScrollDelta::PixelDelta(delta) => CursorEvent::Scroll({
                    if delta.y > 0.0 {
                        1.0
                    } else if delta.y < 0.0 {
                        -1.0
                    } else {
                        0.0
                    }
                }),
            }),
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state,
                        button: MouseButton::Left,
                        ..
                    },
                ..
            } => match state {
                winit::event::ElementState::Pressed => {
                    context.process_input(CursorEvent::ButtonPressed)
                }
                winit::event::ElementState::Released => {
                    context.process_input(CursorEvent::ButtonReleased)
                }
            },
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
