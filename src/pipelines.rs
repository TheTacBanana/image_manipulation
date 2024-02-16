use pollster::FutureExt;

use crate::{texture::load_bytes, vertex::Vertex};

pub struct Pipelines {
    pub layout: wgpu::PipelineLayout,
    pub interpolation: wgpu::RenderPipeline,
    // pub reduction: wgpu::RenderPipeline,
    pub gamma: wgpu::RenderPipeline,
    // pub cross_correlation: wgpu::RenderPipeline,
    pub output: wgpu::RenderPipeline,
}

impl Pipelines {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        bind_groups: &[&wgpu::BindGroupLayout],
    ) -> Self {
        let s_interpolation = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("interpolation"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/interpolation.wgsl").into()),
        });

        let s_gamma = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("output"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/gamma_correction.wgsl").into()),
        });

        let s_output = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("output"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/output.wgsl").into()),
        });

        let layout = Pipelines::create_pipeline_layout(device, bind_groups);

        let interpolation = Pipelines::create_pipeline(device, config, s_interpolation, &layout);
        let gamma = Pipelines::create_pipeline(device, config, s_gamma, &layout);
        let output = Pipelines::create_pipeline(device, config, s_output, &layout);

        Pipelines {
            layout,
            interpolation,
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
        config: &wgpu::SurfaceConfiguration,
        shader: wgpu::ShaderModule,
        layout: &wgpu::PipelineLayout,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
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
