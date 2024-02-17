use std::{iter, mem};

use cgmath::InnerSpace;
use egui::{epaint::text, Checkbox, ComboBox, Slider};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use futures::SinkExt;
use instant::Instant;
use wgpu::{util::DeviceExt, CommandEncoder, TextureView};

use crate::{
    image_display::{ImageDisplay, ScalingMode},
    input::{CursorEvent, InputContext},
    pipelines::Pipelines,
    texture::Texture,
    thread_context::ThreadContext,
    vertex::Vertex,
};

use super::window::Window;

pub struct GraphicsContext {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    // pub pipeline: wgpu::RenderPipeline,
    pub pipelines: Pipelines,
    pub buffers: (wgpu::Buffer, wgpu::Buffer),
    pub texture_sampler: wgpu::Sampler,
    pub texture_layout: wgpu::BindGroupLayout,
    pub image_display: ImageDisplay,
    pub egui: EguiContext,
    pub input: InputContext,
    pub thread: ThreadContext,
    pub array_buffer: wgpu::Buffer,
    pub array_bind_group: wgpu::BindGroup,
}

pub struct EguiContext {
    pub platform: Platform,
    pub render_pass: RenderPass,
    pub last_frame: Instant,
}

pub struct RenderGroup {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}

impl GraphicsContext {
    const VERTICES: &'static [Vertex] = &[
        Vertex::xyz(1.0, 1.0, 0.0),
        Vertex::xyz(1.0, -1.0, 0.0),
        Vertex::xyz(-1.0, -1.0, 0.0),
        Vertex::xyz(-1.0, 1.0, 0.0),
    ];

    const INDICES: &'static [u16] = &[0, 3, 1, 1, 3, 2];

    const LAPLACIAN: &'static [i32] = &[
        -4, -1, 0, -1, -4, -1, 2, 3, 2, -1, 0, 3, 4, 3, 0, -1, 2, 3, 2, -1, -4, -1, 0, -1, -4,
    ];

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

        let image_display = ImageDisplay::from_window(&device, &window.raw);

        let texture_layout = Texture::create_bind_group_layout(&device);
        let texture_sampler = Texture::create_sampler(&device);

        let array_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        (mem::size_of::<i32>() * Self::LAPLACIAN.len()) as _,
                    ),
                },
                count: None,
            }],
            label: Some("array_bind_group_layout"),
        });

        let array_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("image_display_buf"),
            contents: bytemuck::cast_slice(&Self::LAPLACIAN),
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });

        let array_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &array_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: array_buffer.as_entire_binding(),
            }],
            label: Some("image_display_bind_group"),
        });

        let pipelines = Pipelines::new(
            &device,
            &config,
            &[&texture_layout, &image_display.layout, &array_layout],
        );

        let buffers = Self::create_buffers(&device);

        Self {
            surface,
            device,
            queue,
            config,
            pipelines,
            buffers,
            texture_sampler,
            texture_layout,
            image_display,
            egui,
            input: InputContext::default(),
            thread: ThreadContext::default(),
            array_buffer,
            array_bind_group,
        }
    }

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

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.image_display.window_size = [width as f32, height as f32];
        }
    }

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
                (texture.texture.width() as f32 * self.image_display.size).floor() as u32,
            ),
            u32::max(
                1,
                (texture.texture.height() as f32 * self.image_display.size).floor() as u32,
            ),
        );

        let mut render_groups = (0..2)
            .map(|_| self.create_render_group(texture_dims))
            .collect::<Vec<_>>();

        // Interpolate image
        self.render_pass(
            &mut encoder,
            &self.pipelines.interpolation,
            &texture.bind_group,
            &render_groups[0].view,
            true,
        );

        if self.image_display.cross_correlation {
            // Apply kernel to interpolated image
            self.render_pass(
                &mut encoder,
                &self.pipelines.kernel,
                &render_groups[0].bind_group,
                &render_groups[1].view,
                false,
            );
            render_groups.reverse();

            let size = u32::max(
                texture_dims.0.next_power_of_two(),
                texture_dims.0.next_power_of_two(),
            );
            let padded = self.create_render_group((size, size));

            // Pad the image to a power of 2
            self.render_pass(
                &mut encoder,
                &self.pipelines.pad,
                &render_groups[0].bind_group,
                &padded.view,
                false,
            );

            let mut reductions = vec![padded];

            (0..size.ilog2()).rev().for_each(|i| {
                let m = 2u32.pow(i);
                let dims = (m, m);

                reductions.push(self.create_render_group(dims));
            });

            reductions.windows(2).for_each(|reductions| {
                self.render_pass(
                    &mut encoder,
                    &self.pipelines.reduction,
                    &reductions[0].bind_group,
                    &reductions[1].view,
                    false,
                )
            })

            // self.render_pass(
            //     &mut encoder,
            //     &self.pipelines.normalize,
            //     &render_groups[0].bind_group,
            //     &render_groups[1].view,
            //     false,
            // );
            // render_groups.reverse()
        }

        // Gamma correct the interpolated image
        self.render_pass(
            &mut encoder,
            &self.pipelines.gamma,
            &render_groups[0].bind_group,
            &render_groups[1].view,
            false,
        );
        render_groups.reverse();

        // Get current screen texture
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Render the modified tex to screenspace
        self.render_pass(
            &mut encoder,
            &self.pipelines.output,
            &render_groups[0].bind_group,
            &view,
            true,
        );

        // Render UI
        self.render_egui(&mut encoder, &view, window);

        // Submit all work to queue and present
        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        self.egui.last_frame = Instant::now();

        Ok(())
    }

    pub fn create_render_group(&self, dims: (u32, u32)) -> RenderGroup {
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: dims.0,
                height: dims.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.texture_sampler),
                },
            ],
            label: None,
        });
        RenderGroup {
            texture: tex,
            view,
            bind_group,
        }
    }

    pub fn render_pass(
        &self,
        encoder: &mut CommandEncoder,
        pipeline: &wgpu::RenderPipeline,
        tex_in: &wgpu::BindGroup,
        tex_out: &wgpu::TextureView,
        clear: bool,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &tex_out,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: {
                        match clear {
                            true => wgpu::LoadOp::Clear(wgpu::Color {
                                r: self.image_display.background_colour[0] as f64,
                                g: self.image_display.background_colour[1] as f64,
                                b: self.image_display.background_colour[2] as f64,
                                a: self.image_display.background_colour[3] as f64,
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

        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, &tex_in, &[]);
        render_pass.set_bind_group(1, &self.image_display.bind_group, &[]);
        render_pass.set_bind_group(2, &self.array_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.buffers.0.slice(..));
        render_pass.set_index_buffer(self.buffers.1.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..GraphicsContext::INDICES.len() as u32, 0, 0..1);
    }

    pub fn render_egui(
        &mut self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        window: &winit::window::Window,
    ) {
        self.egui
            .platform
            .update_time(self.egui.last_frame.elapsed().as_secs_f64());

        self.egui.platform.begin_frame();

        let ctx = &self.egui.platform.context();

        egui::Window::new("Image Settings")
            .collapsible(false)
            .show(ctx, |ui| {
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

                {
                    let mut x_pos = self.image_display.pos[0].to_string();
                    let mut y_pos = self.image_display.pos[1].to_string();

                    ui.add(egui::TextEdit::singleline(&mut x_pos));
                    ui.add(egui::TextEdit::singleline(&mut y_pos));

                    if let Ok(x_pos) = x_pos.parse::<f32>() {
                        self.image_display.pos[0] = x_pos;
                    }
                    if let Ok(y_pos) = y_pos.parse::<f32>() {
                        self.image_display.pos[1] = y_pos;
                    }
                }

                ui.add(
                    Slider::new(&mut self.image_display.gamma, 0.0..=5.0).text("Gamma Correction"),
                );

                ui.add(Slider::new(&mut self.image_display.size, 0.0..=10.0).text("Image Size"));

                ComboBox::from_label("")
                    .selected_text(format!("{:?}", &mut self.image_display.scaling_mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.image_display.scaling_mode,
                            ScalingMode::NearestNeighbour,
                            "Nearest Neighbour",
                        );
                        ui.selectable_value(
                            &mut self.image_display.scaling_mode,
                            ScalingMode::Bilinear,
                            "Bi-Linear",
                        );
                    });

                ui.add(Checkbox::new(
                    &mut self.image_display.cross_correlation,
                    "Cross Correlation",
                ));

                if ui.button("Reset Default").clicked() {
                    self.image_display.reset_default();
                }

                {
                    let colour = &mut self.image_display.background_colour;
                    let mut rgb = [colour[0], colour[1], colour[2]];
                    egui::color_picker::color_edit_button_rgb(ui, &mut rgb);
                    colour[0] = rgb[0];
                    colour[1] = rgb[1];
                    colour[2] = rgb[2];
                }

                self.input.mouse_over_ui = ui.ui_contains_pointer();
            });

        let full_output = self.egui.platform.end_frame(Some(window));
        let paint_jobs = self.egui.platform.context().tessellate(full_output.shapes);

        let screen_descriptor = ScreenDescriptor {
            physical_width: self.config.width,
            physical_height: self.config.height,
            scale_factor: window.scale_factor() as f32,
        };
        let tdelta = full_output.textures_delta;
        self.egui
            .render_pass
            .add_textures(&self.device, &self.queue, &tdelta)
            .expect("add texture ok");
        self.egui.render_pass.update_buffers(
            &self.device,
            &self.queue,
            &paint_jobs,
            &screen_descriptor,
        );

        self.egui
            .render_pass
            .execute(encoder, view, &paint_jobs, &screen_descriptor, None)
            .unwrap();

        self.egui
            .render_pass
            .remove_textures(tdelta)
            .expect("remove texture ok");
    }

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
                            self.image_display.pos[0] += delta.x;
                            self.image_display.pos[1] += delta.y;
                        }
                        2 => {
                            let ts = input.active_touches();
                            let (one, two) = (ts[0], ts[1]);

                            let between = two - one;
                            let m1 = between.magnitude();
                            let m2 = (between + delta).magnitude();

                            self.image_display.size -= (m2 / m1) - 1.0;
                            self.image_display.size = f32::max(self.image_display.size, 0.001);
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
                    self.image_display.pos[0] += pos.x - input.last_mouse_pos.x;
                    self.image_display.pos[1] += pos.y - input.last_mouse_pos.y;
                }
                input.last_mouse_pos = pos;
            }
            CursorEvent::Scroll(scroll) => {
                self.image_display.size +=
                    scroll * (self.image_display.size * self.image_display.size + 1.1).log10();
                self.image_display.size = f32::min(f32::max(self.image_display.size, 0.001), 10.0);
            }
        }
    }
}
