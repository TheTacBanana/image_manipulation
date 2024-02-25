use std::{iter, mem};

use cgmath::InnerSpace;
use egui::{epaint::text, Checkbox, ComboBox, Slider};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use futures::SinkExt;
use image::EncodableLayout;
use instant::Instant;
use wgpu::{util::DeviceExt, BindGroupDescriptor, CommandEncoder, TextureView};

use crate::{
    image_display::{ImageDisplay, ImageDisplayWithBuffers, ScalingMode},
    input::{CursorEvent, InputContext},
    pipelines::Pipelines,
    stages::{RenderGroup, RenderStages},
    texture::Texture,
    thread_context::ThreadContext,
    vertex::Vertex,
};

use super::window::Window;

// Graphical conext containing all data
pub struct GraphicsContext {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub pipelines: Pipelines,
    pub stages: RenderStages,
    pub buffers: (wgpu::Buffer, wgpu::Buffer),
    pub texture_sampler: wgpu::Sampler,
    pub image_display: ImageDisplayWithBuffers,
    pub egui: EguiContext,
    pub input: InputContext,
    pub thread: ThreadContext,
    pub kernel_render_group: RenderGroup,
}

// Context containing egui related items
pub struct EguiContext {
    pub platform: Platform,
    pub render_pass: RenderPass,
    pub last_frame: Instant,
}

impl GraphicsContext {
    // Vertexes spanning screenspace
    const VERTICES: &'static [Vertex] = &[
        Vertex::xyz(1.0, 1.0, 0.0),
        Vertex::xyz(1.0, -1.0, 0.0),
        Vertex::xyz(-1.0, -1.0, 0.0),
        Vertex::xyz(-1.0, 1.0, 0.0),
    ];

    // Indices for vertexes
    const INDICES: &'static [u16] = &[0, 3, 1, 1, 3, 2];

    // Laplacian matrix
    pub const LAPLACIAN: &'static [f32; 25] = &[
        -4.0, -1.0, 0.0, -1.0, -4.0, -1.0, 2.0, 3.0, 2.0, -1.0, 0.0, 3.0, 4.0, 3.0, 0.0, -1.0, 2.0,
        3.0, 2.0, -1.0, -4.0, -1.0, 0.0, -1.0, -4.0,
    ];

    // Create a new graphics contexts
    pub async fn new(window: &Window) -> Self {
        let size = window.raw.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window.raw) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    #[cfg(target_arch = "wasm32")]
                    limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    #[cfg(not(target_arch = "wasm32"))]
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let egui = EguiContext {
            platform: Platform::new(PlatformDescriptor {
                physical_width: size.width,
                physical_height: size.height,
                scale_factor: window.raw.scale_factor(),
                ..Default::default()
            }),
            render_pass: RenderPass::new(&device, surface_format, 1),
            last_frame: Instant::now(),
        };

        let image_display = ImageDisplayWithBuffers::from_window(&device, &window.raw);
        let texture_sampler = Texture::create_sampler(&device);
        let pipelines = Pipelines::new(&device, surface_format, &image_display.layout).await;
        let stages = RenderStages::new();
        let buffers = GraphicsContext::create_buffers(&device);

        let kernel_render_group = RenderGroup::new_without_context(
            (5, 5),
            &device,
            wgpu::TextureFormat::Rgba32Float,
            &texture_sampler,
            &pipelines,
        );
        GraphicsContext::write_kernel_texture(
            &queue,
            &kernel_render_group.texture,
            &GraphicsContext::LAPLACIAN,
        );

        Self {
            surface,
            device,
            queue,
            config,
            pipelines,
            stages,
            buffers,
            texture_sampler,
            image_display,
            egui,
            input: InputContext::default(),
            thread: ThreadContext::default(),
            kernel_render_group,
        }
    }

    // Create vertex and index buffers
    pub fn create_buffers(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex_buf"),
            contents: bytemuck::cast_slice(GraphicsContext::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index_buf"),
            contents: bytemuck::cast_slice(GraphicsContext::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        (vertex_buffer, index_buffer)
    }

    pub fn write_kernel_texture(queue: &wgpu::Queue, texture: &wgpu::Texture, data: &[f32; 25]) {
        let mut normalized_values = Vec::new();
        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &{
                for i in data {
                    let rgba = [
                        f32::max(0.0, f32::min(1.0, (*i / 256.0) + 0.5)),
                        0.0,
                        0.0,
                        0.0,
                    ];
                    normalized_values.extend_from_slice(&rgba);
                }
                normalized_values.as_bytes()
            },
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * 4 * 5),
                rows_per_image: Some(5),
            },
            texture.size(),
        );
    }

    // Resize window callback
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.image_display.internal.window_size = [width as f32, height as f32];
        }
    }

    pub fn image_display(&self) -> &ImageDisplay {
        &self.image_display.internal
    }

    pub fn image_display_mut(&mut self) -> &mut ImageDisplay {
        &mut self.image_display.internal
    }

    // Perform all render tasks per frame
    pub fn render(
        &mut self,
        texture: &Texture,
        window: &winit::window::Window,
    ) -> Result<(), wgpu::SurfaceError> {
        self.image_display.bind(self);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let texture_dims = (
            u32::max(
                1,
                (texture.texture.width() as f32 * self.image_display().size).floor() as u32,
            ),
            u32::max(
                1,
                (texture.texture.height() as f32 * self.image_display().size).floor() as u32,
            ),
        );

        if self.image_display.changed {
            let mut stages = mem::take(&mut self.stages);
            stages.update_resolution(&self, texture_dims);
            self.stages = stages;

            // Interpolate image
            self.render_pass(
                &mut encoder,
                &self.pipelines.interpolation,
                &texture.bind_group,
                &self.stages.output_staging().view,
                false,
            );

            self.render_pass(
                &mut encoder,
                &self.pipelines.gamma_lut,
                &self.stages.output_staging().bind_group,
                &self.stages.gamma_lut().view,
                false,
            );

            if self.image_display().cross_correlation {
                // Apply kernel to interpolated image
                self.render_pass(
                    &mut encoder,
                    &self.pipelines.kernel,
                    &self.stages.output_staging().bind_group,
                    &self.stages.kerneled().view,
                    false,
                );

                // Get Min Max from the kernelled image
                self.render_pass(
                    &mut encoder,
                    &self.pipelines.min_max,
                    &self.stages.kerneled().bind_group,
                    &self.stages.min_max().view,
                    false,
                );

                // Normalize the image based on the Min Max found
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &self.stages.output_staging().view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                    render_pass.set_pipeline(&self.pipelines.normalize);
                    render_pass.set_bind_group(0, &self.stages.kerneled().bind_group, &[]);
                    render_pass.set_bind_group(1, &self.image_display.bind_group, &[]);
                    render_pass.set_bind_group(2, &self.kernel_render_group.bind_group, &[]);
                    render_pass.set_bind_group(3, &self.stages.min_max().bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.buffers.0.slice(..));
                    render_pass
                        .set_index_buffer(self.buffers.1.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..GraphicsContext::INDICES.len() as u32, 0, 0..1);
                }
            }
            self.image_display.clear_changed();
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.stages.gamma().view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipelines.gamma);
            render_pass.set_bind_group(0, &self.stages.output_staging().bind_group, &[]);
            render_pass.set_bind_group(1, &self.image_display.bind_group, &[]);
            render_pass.set_bind_group(2, &self.kernel_render_group.bind_group, &[]);
            render_pass.set_bind_group(3, &self.stages.gamma_lut().bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.buffers.0.slice(..));
            render_pass.set_index_buffer(self.buffers.1.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..GraphicsContext::INDICES.len() as u32, 0, 0..1);
        }

        // Get current screen texture
        let output = self.surface.get_current_texture()?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Render the modified tex to screenspace
        self.render_pass(
            &mut encoder,
            &self.pipelines.output,
            &self.stages.gamma().bind_group,
            &output_view,
            true,
        );

        // Render UI
        self.render_egui(&mut encoder, &output_view, window);

        // Submit all work to queue and present
        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        self.egui.last_frame = Instant::now();

        Ok(())
    }

    // Perform a render pass from a bind group, to a texture view
    pub fn render_pass(
        &self,
        encoder: &mut CommandEncoder,
        pipeline: &wgpu::RenderPipeline,
        tex_in: &wgpu::BindGroup,
        tex_out: &wgpu::TextureView,
        clear: bool,
    ) {
        // Begin render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &tex_out,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: {
                        match clear {
                            true => wgpu::LoadOp::Clear(wgpu::Color {
                                r: self.image_display().background_colour[0] as f64,
                                g: self.image_display().background_colour[1] as f64,
                                b: self.image_display().background_colour[2] as f64,
                                a: self.image_display().background_colour[3] as f64,
                            }),
                            false => wgpu::LoadOp::Load,
                        }
                    },
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        // Bind everything and draw
        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, &tex_in, &[]);
        render_pass.set_bind_group(1, &self.image_display.bind_group, &[]);
        render_pass.set_bind_group(2, &self.kernel_render_group.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.buffers.0.slice(..));
        render_pass.set_index_buffer(self.buffers.1.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..GraphicsContext::INDICES.len() as u32, 0, 0..1);
    }

    // Render the ui using egui
    pub fn render_egui(
        &mut self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        window: &winit::window::Window,
    ) {
        // Update the egui frametime
        self.egui
            .platform
            .update_time(self.egui.last_frame.elapsed().as_secs_f64());

        self.egui.platform.begin_frame();

        let cloned = self.image_display().clone();

        // Draw all UI
        let ctx = &self.egui.platform.context();
        egui::Window::new("Image Settings")
            .collapsible(false)
            .show(ctx, |ui| {
                // Open file button
                if (ui.button("Open file")).clicked() {
                    let dialog = rfd::AsyncFileDialog::new()
                        .add_filter("img", &["png", "jpg"])
                        .set_parent(&window)
                        .pick_file();

                    let mut cloned_sender = self.thread.sender.clone();
                    self.thread.execute(async move {
                        let file = dialog.await;

                        if let Some(file) = file {
                            let bytes = file.read().await;
                            cloned_sender.send(bytes).await.unwrap();
                        }
                    });
                }

                // Position Boxes
                {
                    ui.add(egui::DragValue::new(&mut self.image_display_mut().pos[0]).speed(1.0));
                    ui.add(egui::DragValue::new(&mut self.image_display_mut().pos[1]).speed(1.0));
                }

                // Gamma correction slider
                ui.add(
                    Slider::new(&mut self.image_display_mut().gamma, 0.0..=2.0)
                        .text("Gamma Correction"),
                );

                // Image side slider
                ui.add(
                    Slider::new(&mut self.image_display_mut().size, 0.0..=10.0).text("Image Size"),
                );

                // Scaling mode selection box
                ComboBox::from_label("")
                    .selected_text(format!("{:?}", &mut self.image_display_mut().scaling_mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.image_display_mut().scaling_mode,
                            ScalingMode::NearestNeighbour,
                            "Nearest Neighbour",
                        );
                        ui.selectable_value(
                            &mut self.image_display_mut().scaling_mode,
                            ScalingMode::Bilinear,
                            "Bi-Linear",
                        );
                    });

                // Cross correlation
                {
                    ui.add(Checkbox::new(
                        &mut self.image_display_mut().cross_correlation,
                        "Cross Correlation",
                    ));

                    if self.image_display().cross_correlation {
                        ui.separator();

                        for row in 0..5 {
                            ui.horizontal(|ui| {
                                for col in 0..5 {
                                    ui.add(
                                        egui::DragValue::new(
                                            &mut self.image_display_mut().kernel[5 * row + col],
                                        )
                                        .speed(0.01)
                                        .clamp_range(-10.0..=10.0),
                                    );
                                }
                            });
                        }
                        if ui.button("Update").clicked() {
                            GraphicsContext::write_kernel_texture(
                                &self.queue,
                                &self.kernel_render_group.texture,
                                &self.image_display.internal.kernel,
                            );
                            self.image_display.set_changed();
                        }
                        ui.separator();
                    }
                }

                // Background colour wheel
                {
                    let colour = &mut self.image_display_mut().background_colour;
                    let mut rgb = [colour[0], colour[1], colour[2]];
                    egui::color_picker::color_edit_button_rgb(ui, &mut rgb);
                    colour[0] = rgb[0];
                    colour[1] = rgb[1];
                    colour[2] = rgb[2];
                }

                // Reset to defaults button
                if ui.button("Reset Default").clicked() {
                    self.image_display_mut().reset_default();
                    GraphicsContext::write_kernel_texture(
                        &self.queue,
                        &self.kernel_render_group.texture,
                        &self.image_display.internal.kernel,
                    );
                }

                self.input.mouse_over_ui = ui.ui_contains_pointer();
            });

        // Check if has changed
        if *self.image_display() != cloned {
            self.image_display.set_changed()
        }

        let full_output = self.egui.platform.end_frame(Some(window));
        let paint_jobs = self.egui.platform.context().tessellate(full_output.shapes);

        let screen_descriptor = ScreenDescriptor {
            physical_width: self.config.width,
            physical_height: self.config.height,
            scale_factor: window.scale_factor() as f32,
        };
        let tdelta = full_output.textures_delta;

        let render_pass = &mut self.egui.render_pass;
        render_pass
            .add_textures(&self.device, &self.queue, &tdelta)
            .unwrap();
        render_pass.update_buffers(&self.device, &self.queue, &paint_jobs, &screen_descriptor);
        render_pass
            .execute(encoder, view, &paint_jobs, &screen_descriptor, None)
            .unwrap();
        render_pass.remove_textures(tdelta).unwrap();
    }

    // Process a cursor event
    pub fn process_input(&mut self, event: CursorEvent) {
        let input = &mut self.input;
        match event {
            CursorEvent::StartTouch(id, pos) => {
                input.start_touch(id, pos);
            }
            CursorEvent::TouchMove(id, pos) => {
                let delta = input.update_touch(id, pos).unwrap();

                if !input.mouse_over_ui {
                    match input.touch_count() {
                        1 => {
                            self.image_display_mut().pos[0] += delta.x;
                            self.image_display_mut().pos[1] += delta.y;
                        }
                        2 => {
                            let ts = input.active_touches();
                            let (one, two) = (ts[0], ts[1]);

                            let between = two - one;
                            let m1 = between.magnitude();
                            let m2 = (between + delta).magnitude();

                            self.image_display_mut().size -= (m2 / m1) - 1.0;
                            self.image_display_mut().size =
                                f32::max(self.image_display().size, 0.001);
                        }
                        _ => (),
                    }
                }
            }
            CursorEvent::EndTouch(id) => {
                input.end_touch(id);
            }
            CursorEvent::ButtonPressed => input.mouse_pressed = true && !input.mouse_over_ui,
            CursorEvent::ButtonReleased => {
                input.mouse_pressed = false;
            }
            CursorEvent::Position(pos) => {
                if input.mouse_pressed {
                    self.image_display.internal.pos[0] += pos.x - input.last_mouse_pos.x;
                    self.image_display.internal.pos[1] += pos.y - input.last_mouse_pos.y;
                    // self.image_display.set_changed()
                }
                input.last_mouse_pos = pos;
            }
            CursorEvent::Scroll(scroll) => {
                self.image_display_mut().size +=
                    scroll * (self.image_display().size * self.image_display().size + 1.1).log10();
                self.image_display_mut().size =
                    f32::min(f32::max(self.image_display().size, 0.001), 10.0);
                self.image_display.set_changed()
            }
        }
    }
}
