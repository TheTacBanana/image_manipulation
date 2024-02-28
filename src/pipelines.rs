use crate::{load_bytes, vertex::Vertex};

/// Pipelines created from shaders
pub struct Pipelines {
    pub bind_group_layouts: TextureBindGroupLayouts,
    pub interpolation: wgpu::RenderPipeline,
    pub kernel: wgpu::RenderPipeline,
    pub min_max: wgpu::RenderPipeline,
    pub normalize: wgpu::RenderPipeline,
    pub gamma_lut: wgpu::RenderPipeline,
    pub gamma: wgpu::RenderPipeline,
    pub output: wgpu::RenderPipeline,
}

/// Bind group layouts for textures
pub struct TextureBindGroupLayouts {
    pub bgra8unormsrgb: wgpu::BindGroupLayout,
    pub rgba32float: wgpu::BindGroupLayout,
}

impl TextureBindGroupLayouts {
    /// Create bind groups for filtered and non filtered texture layouts
    pub fn create(device: &wgpu::Device) -> Self {
        let bgra8unormsrgb = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let rgba32float = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
            label: Some("non_filter_texture_bind_group_layout"),
        });

        TextureBindGroupLayouts {
            bgra8unormsrgb,
            rgba32float,
        }
    }
}

/// Bind a bind group to a given index
pub struct Binding<'a>(pub u32, pub &'a wgpu::BindGroup);

impl Pipelines {
    /// Create a new Pipelines struct and load all shaders
    pub async fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        image_display_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        // Create Texture Bind Group Layouts
        let layouts = TextureBindGroupLayouts::create(device);

        // Load Shaders
        let s_interp = Pipelines::load_shader(device, "./src/shader/interpolation.wgsl").await;
        let s_kernel = Pipelines::load_shader(device, "./src/shader/kernel.wgsl").await;
        let s_for_loop = Pipelines::load_shader(device, "./src/shader/min_max.wgsl").await;
        let s_normalize = Pipelines::load_shader(device, "./src/shader/normalize.wgsl").await;
        let s_gamma_lut = Pipelines::load_shader(device, "./src/shader/gamma_lookup.wgsl").await;
        let s_gamma = Pipelines::load_shader(device, "./src/shader/gamma_correction.wgsl").await;
        let s_output = Pipelines::load_shader(device, "./src/shader/output.wgsl").await;

        // Create Pipeline Layouts
        let normal_layout = Pipelines::create_pipeline_layout(
            device,
            &[
                &layouts.rgba32float,
                image_display_layout,
                &layouts.rgba32float,
            ],
        );

        // Create Pipelines
        let interpolation = Pipelines::create_pipeline(
            device,
            s_interp,
            &Pipelines::create_pipeline_layout(
                device,
                &[&layouts.bgra8unormsrgb, image_display_layout],
            ),
            wgpu::TextureFormat::Rgba32Float,
            "interpolation",
        );
        let kernel = Pipelines::create_pipeline(
            device,
            s_kernel,
            &normal_layout,
            wgpu::TextureFormat::Rgba32Float,
            "kernel",
        );
        let min_max = Pipelines::create_pipeline(
            device,
            s_for_loop,
            &normal_layout,
            wgpu::TextureFormat::Rgba32Float,
            "min_max",
        );
        let normalize = Pipelines::create_pipeline(
            device,
            s_normalize,
            &normal_layout,
            wgpu::TextureFormat::Rgba32Float,
            "normalize",
        );
        let gamma_lut = Pipelines::create_pipeline(
            device,
            s_gamma_lut,
            &Pipelines::create_pipeline_layout(device, &[&image_display_layout]),
            wgpu::TextureFormat::Rgba32Float,
            "gamma_lut",
        );
        let gamma = Pipelines::create_pipeline(
            device,
            s_gamma,
            &normal_layout,
            wgpu::TextureFormat::Rgba32Float,
            "gamma",
        );
        let output = Pipelines::create_pipeline(
            device,
            s_output,
            &Pipelines::create_pipeline_layout(
                device,
                &[&layouts.rgba32float, &image_display_layout],
            ),
            output_format,
            "output",
        );

        // Return pipelines struct
        Pipelines {
            bind_group_layouts: layouts,
            interpolation,
            kernel,
            min_max,
            normalize,
            gamma_lut,
            gamma,
            output,
        }
    }

    /// Load shader bytes and create a shader module
    async fn load_shader(device: &wgpu::Device, path: &str) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(path),
            source: wgpu::ShaderSource::Wgsl(String::from_utf8_lossy(
                &load_bytes(path).await.unwrap(),
            )),
        })
    }

    /// Create a layout from a list of bind groups
    fn create_pipeline_layout(
        device: &wgpu::Device,
        bind_groups: &[&wgpu::BindGroupLayout],
    ) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render_pipeline_layout"),
            bind_group_layouts: bind_groups,
            push_constant_ranges: &[],
        })
    }

    /// Create a pipeline with a shader, layout and format it is rendering to
    fn create_pipeline(
        device: &wgpu::Device,
        shader: wgpu::ShaderModule,
        layout: &wgpu::PipelineLayout,
        target_format: wgpu::TextureFormat,
        label: &str,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: match target_format {
                        wgpu::TextureFormat::Rgba32Float => None,
                        _ => Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                    },
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }
}
