#![feature(async_closure)]
#![feature(option_take_if)]

use context::GraphicsContext;

use futures::SinkExt;
use image::EncodableLayout;
use input::CursorEvent;
use window::Window;
use winit::{
    event::{Event, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::ControlFlow,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub mod context;
pub mod image_display;
pub mod input;
pub mod pipelines;
pub mod stages;
pub mod thread_context;
pub mod vertex;
pub mod window;

use anyhow::Result;
use cfg_if::cfg_if;

pub async fn load_bytes(path: &str) -> Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let window = web_sys::window().unwrap();
            let origin = window.origin();
            let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
            let path = base.join(path).unwrap();
            let bytes = reqwest::get(path)
                .await?
                .bytes()
                .await?;
        } else {
            let bytes = std::fs::read(path)?;
        }
    }
    Ok(bytes.to_vec())
}

// Entry point for the program
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    // Create a window and graphics context
    let window = Window::new();
    let mut context = GraphicsContext::new(&window).await;

    window.run(move |window, event, control_flow| {
        // Load a new image if bytes receieved from the channel
        if let Ok(Some(bytes)) = context.thread.receiver.try_next() {
            let _ = context.load_texture(bytes.as_bytes());
        }

        // Handle Winit Events
        context.egui.platform.handle_event(&event);
        match event {
            Event::RedrawRequested(_) => {
                context.render(window).unwrap();
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
                let mut sender = context.thread.sender.clone();
                context.thread.execute(async move {
                    let bytes = load_bytes(path.to_str().unwrap()).await;
                    if let Ok(bytes) = bytes {
                        let _ = sender.send(bytes).await;
                    }
                })
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
