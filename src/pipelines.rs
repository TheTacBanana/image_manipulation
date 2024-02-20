use pollster::FutureExt;

use crate::{
    texture::{load_bytes, Texture},
    vertex::Vertex,
};

pub struct Pipelines {
    pub bind_group_layouts: TextureBindGroupLayouts,

    pub interpolation: wgpu::RenderPipeline,
    pub kernel: wgpu::RenderPipeline,
    pub min_max: wgpu::RenderPipeline,
    pub normalize: wgpu::RenderPipeline,
    pub gamma: wgpu::RenderPipeline,
    pub output: wgpu::RenderPipeline,
}

pub struct TextureBindGroupLayouts {
    pub bgra8unormsrgb: wgpu::BindGroupLayout,
    pub rgba32float: wgpu::BindGroupLayout,
}

impl Pipelines {
    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        image_display_layout: &wgpu::BindGroupLayout,
        kernel_array_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let layouts = TextureBindGroupLayouts {
            bgra8unormsrgb: Texture::create_bind_group_layout(device),
            rgba32float: Texture::create_non_filter_bind_group_layout(device),
        };

        let s_interpolation = Pipelines::load_shader(device, "./src/shader/interpolation.wgsl");
        let s_kernel = Pipelines::load_shader(device, "./src/shader/kernel.wgsl");
        let s_for_loop = Pipelines::load_shader(device, "./src/shader/min_max.wgsl");
        let s_normalize = Pipelines::load_shader(device, "./src/shader/normalize.wgsl");
        let s_gamma = Pipelines::load_shader(device, "./src/shader/gamma_correction.wgsl");
        let s_output = Pipelines::load_shader(device, "./src/shader/output.wgsl");

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

        let interpolation = Pipelines::create_pipeline(
            device,
            s_interpolation,
            &interpolation_layout,
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
        let for_loop = Pipelines::create_pipeline(
            device,
            s_for_loop,
            &normal_layout,
            wgpu::TextureFormat::Rgba32Float,
            "for_loop",
        );
        let normalize = Pipelines::create_pipeline(
            device,
            s_normalize,
            &normalisation_layout,
            wgpu::TextureFormat::Rgba32Float,
            "normalize",
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
            &normal_layout,
            output_format,
            "output",
        );

        Pipelines {
            bind_group_layouts: layouts,
            interpolation,
            kernel,
            min_max: for_loop,
            normalize,
            gamma,
            output,
        }
    }

    fn load_shader(device: &wgpu::Device, path: &str) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(path),
            source: wgpu::ShaderSource::Wgsl(String::from_utf8_lossy(
                &load_bytes(path).block_on().unwrap(),
            )),
        })
    }

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
