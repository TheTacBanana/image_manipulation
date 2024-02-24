use pollster::FutureExt;

use crate::{
    texture::{load_bytes, Texture},
    vertex::Vertex,
};

// Pipelines created from shaders
pub struct Pipelines {
    pub bind_group_layouts: TextureBindGroupLayouts,
    pub pipeline_layouts: PipelineLayouts,
    pub interpolation: wgpu::RenderPipeline,
    pub kernel: wgpu::RenderPipeline,
    pub min_max: wgpu::RenderPipeline,
    pub normalize: wgpu::RenderPipeline,
    pub gamma: wgpu::RenderPipeline,
    pub output: wgpu::RenderPipeline,
}

// Bind group layouts for textures
pub struct TextureBindGroupLayouts {
    pub bgra8unormsrgb: wgpu::BindGroupLayout,
    pub rgba32float: wgpu::BindGroupLayout,
}

pub struct PipelineLayouts {
    interpolation: wgpu::PipelineLayout,
    normal: wgpu::PipelineLayout,
    normalisation: wgpu::PipelineLayout,
}

impl Pipelines {
    // Create a new Pipelines struct and load all shaders
    pub async fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        image_display_layout: &wgpu::BindGroupLayout,
        kernel_array_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        // Create Texture Bind Group Layouts
        let layouts = TextureBindGroupLayouts {
            bgra8unormsrgb: Texture::create_bind_group_layout(device),
            rgba32float: Texture::create_non_filter_bind_group_layout(device),
        };

        // Load Shaders
        let s_interpolation = Pipelines::load_shader(device, "./src/shader/interpolation.wgsl").await;
        let s_kernel = Pipelines::load_shader(device, "./src/shader/kernel.wgsl").await;
        let s_for_loop = Pipelines::load_shader(device, "./src/shader/min_max.wgsl").await;
        let s_normalize = Pipelines::load_shader(device, "./src/shader/normalize.wgsl").await;
        let s_gamma = Pipelines::load_shader(device, "./src/shader/gamma_correction.wgsl").await;
        let s_output = Pipelines::load_shader(device, "./src/shader/output.wgsl").await;

        // Create Pipeline Layouts
        let interpolation_layout = Pipelines::create_pipeline_layout(
            device,
            &[
                &layouts.bgra8unormsrgb,
                image_display_layout,
                kernel_array_layout,
            ],
        );
        let normal_layout = Pipelines::create_pipeline_layout(
            device,
            &[
                &layouts.rgba32float,
                image_display_layout,
                kernel_array_layout,
            ],
        );
        let normalisation_layout = Pipelines::create_pipeline_layout(
            device,
            &[
                &layouts.rgba32float,
                image_display_layout,
                kernel_array_layout,
                &layouts.rgba32float,
            ],
        );

        let pipeline_layouts = PipelineLayouts {
            interpolation: interpolation_layout,
            normal: normal_layout,
            normalisation: normalisation_layout,
        };

        // Create Pipelines
        let interpolation = Pipelines::create_pipeline(
            device,
            s_interpolation,
            &pipeline_layouts.interpolation,
            wgpu::TextureFormat::Rgba32Float,
            "interpolation",
        );
        let kernel = Pipelines::create_pipeline(
            device,
            s_kernel,
            &pipeline_layouts.normal,
            wgpu::TextureFormat::Rgba32Float,
            "kernel",
        );
        let for_loop = Pipelines::create_pipeline(
            device,
            s_for_loop,
            &pipeline_layouts.normal,
            wgpu::TextureFormat::Rgba32Float,
            "for_loop",
        );
        let normalize = Pipelines::create_pipeline(
            device,
            s_normalize,
            &pipeline_layouts.normalisation,
            wgpu::TextureFormat::Rgba32Float,
            "normalize",
        );
        let gamma = Pipelines::create_pipeline(
            device,
            s_gamma,
            &pipeline_layouts.normal,
            wgpu::TextureFormat::Rgba32Float,
            "gamma",
        );
        let output = Pipelines::create_pipeline(
            device,
            s_output,
            &pipeline_layouts.normal,
            output_format,
            "output",
        );

        // Return pipelines struct
        Pipelines {
            pipeline_layouts,
            bind_group_layouts: layouts,
            interpolation,
            kernel,
            min_max: for_loop,
            normalize,
            gamma,
            output,
        }
    }

    // Load shader bytes and create a shader module
    async fn load_shader(device: &wgpu::Device, path: &str) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(path),
            source: wgpu::ShaderSource::Wgsl(String::from_utf8_lossy(
                &load_bytes(path).await.unwrap(),
            )),
        })
    }

    // Create a layout from a list of bind groups
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

    // Create a pipeline with a shader, layout and format it is rendering to
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

    pub fn hot_load_interpolation(&mut self, device: &wgpu::Device) {
        let s_interpolation = Pipelines::load_shader(device, "./src/shader/interpolation.wgsl").block_on();

        self.interpolation = Pipelines::create_pipeline(
            device,
            s_interpolation,
            &self.pipeline_layouts.interpolation,
            wgpu::TextureFormat::Rgba32Float,
            "interpolation",
        );
    }
}
