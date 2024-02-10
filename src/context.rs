use std::iter;

use cgmath::{InnerSpace, MetricSpace, Vector2, Zero};
use egui::{Checkbox, ComboBox, Slider, TextBuffer};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use instant::Instant;
use wgpu::{util::DeviceExt, CommandEncoder, TextureView};

use crate::{
    input::{CursorEvent, InputContext},
    texture::Texture,
    vertex::Vertex,
    viewport::{ImageDisplay, ScalingMode, ViewportDimensions},
};

use super::window::Window;

const VERTICES: &[Vertex] = &[
    Vertex::xyz(1.0, 1.0, 0.0),
    Vertex::xyz(1.0, -1.0, 0.0),
    Vertex::xyz(-1.0, -1.0, 0.0),
    Vertex::xyz(-1.0, 1.0, 0.0),
];

const INDICES: &[u16] = &[0, 3, 1, 1, 3, 2];

pub struct GraphicsContext {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub dim_buffer: wgpu::Buffer,
    pub dim_bind_group: wgpu::BindGroup,
    pub texture_layout: wgpu::BindGroupLayout,
    pub image_display: ImageDisplay,
    pub image_display_buffer: wgpu::Buffer,
    pub image_display_bind_group: wgpu::BindGroup,
    pub last_frame: Instant,
    pub egui: EguiContext,
    pub input: InputContext,
}

pub struct EguiContext {
    pub platform: Platform,
    pub render_pass: RenderPass,
}

impl GraphicsContext {
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
                    // WebGL doesn't support all of wgpu's features
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
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
            .filter(|f| f.is_srgb())
            .next()
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

        let platform = Platform::new(PlatformDescriptor {
            physical_width: size.width as u32,
            physical_height: size.height as u32,
            scale_factor: window.raw.scale_factor(),
            ..Default::default()
        });

        let render_pass = RenderPass::new(&device, surface_format, 1);

        let dims = ViewportDimensions::from_window(&window.raw);

        let dim_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("dim_bind_group_layout"),
        });

        let dim_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("dim_buf"),
            contents: bytemuck::cast_slice(&[dims]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let dim_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &dim_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: dim_buffer.as_entire_binding(),
            }],
            label: Some("dim_bind_group"),
        });

        let image_display = ImageDisplay::default();

        let image_display_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("image_display_bind_group_layout"),
            });

        let image_display_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("image_display_buf"),
            contents: bytemuck::bytes_of(&image_display.into_raw()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let image_display_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &image_display_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &image_display_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &image_display_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &image_display_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &image_display_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &image_display_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &image_display_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
            label: Some("image_display_bind_group"),
        });

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&dim_layout, &texture_layout, &image_display_layout],
                push_constant_ranges: &[],
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex_buf"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index_buf"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let last_frame = Instant::now();

        Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            vertex_buffer,
            index_buffer,
            dim_buffer,
            dim_bind_group,
            texture_layout,
            image_display,
            image_display_buffer,
            image_display_bind_group,
            last_frame,
            egui: EguiContext {
                platform,
                render_pass,
            },
            input: InputContext::default(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            ViewportDimensions::from_dim(width, height).bind(&self)
        }
    }

    pub fn render(
        &mut self,
        texture: &Texture,
        window: &winit::window::Window,
    ) -> Result<(), wgpu::SurfaceError> {
        {
            let mut take = std::mem::take(&mut self.image_display);
            take.bind(&self);
            self.image_display = take;
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.render_image(&mut encoder, &view, texture);
        self.render_egui(&mut encoder, &view, window);

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        self.last_frame = Instant::now();

        Ok(())
    }

    pub fn render_image(
        &self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        texture: &Texture,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.dim_bind_group, &[]);
        render_pass.set_bind_group(1, &texture.bind_group, &[]);
        render_pass.set_bind_group(2, &self.image_display_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
    }

    pub fn render_egui(
        &mut self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        window: &winit::window::Window,
    ) {
        self.egui
            .platform
            .update_time(self.last_frame.elapsed().as_secs_f64());

        self.egui.platform.begin_frame();

        let ctx = &self.egui.platform.context();

        let mut x_pos = self.image_display.pos[0].to_string();
        let mut y_pos = self.image_display.pos[1].to_string();

        egui::Window::new("Image Settings")
            .collapsible(false)
            .show(ctx, |ui| {
                ui.add(egui::TextEdit::singleline(&mut x_pos));
                ui.add(egui::TextEdit::singleline(&mut y_pos));
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

                self.input.mouse_over_ui = ui.ui_contains_pointer();
            });

        if let Ok(x_pos) = x_pos.parse::<f32>() {
            self.image_display.pos[0] = x_pos;
        }
        if let Ok(y_pos) = y_pos.parse::<f32>() {
            self.image_display.pos[1] = y_pos;
        }

        // End the UI frame. We could now handle the output and draw the UI with the backend.
        let full_output = self.egui.platform.end_frame(Some(&window));
        let paint_jobs = self.egui.platform.context().tessellate(full_output.shapes);

        // Upload all resources for the GPU.
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

        // Record all render passes.
        self.egui
            .render_pass
            .execute(encoder, &view, &paint_jobs, &screen_descriptor, None)
            .unwrap();

        // Submit the commands.
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

                            self.image_display.size -= ((m2 / m1) - 1.0);
                            self.image_display.size = f32::max(self.image_display.size, 0.001);
                        }
                        _ => ()
                    }
                }
            },
            CursorEvent::EndTouch(id) => {
                input.end_touch(id);
            }
            CursorEvent::ButtonPressed => {
                input.mouse_pressed = true && !input.mouse_over_ui
            }
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
                self.image_display.size = f32::max(self.image_display.size, 0.001);
            }
        }
    }
}
