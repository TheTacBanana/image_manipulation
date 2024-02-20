use crate::context::GraphicsContext;

// Wrapper struct around a render target and source
pub struct RenderGroup {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
}

impl RenderGroup {
    // Create a new Render Group with texture, view and bindgroup
    pub fn new(
        context: &GraphicsContext,
        dims: (u32, u32),
        format: wgpu::TextureFormat,
    ) -> RenderGroup {
        let tex = context.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: dims.0,
                height: dims.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = tex.create_view(&wgpu::TextureViewDescriptor {
            format: Some(format),
            ..Default::default()
        });

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: match format {
                    wgpu::TextureFormat::Rgba32Float => {
                        &context.pipelines.bind_group_layouts.rgba32float
                    }
                    wgpu::TextureFormat::Bgra8UnormSrgb => {
                        &context.pipelines.bind_group_layouts.bgra8unormsrgb
                    }
                    _ => panic!(),
                },
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&context.texture_sampler),
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
}

// Wrapper around multiple Render Groups for to ensure the correct
#[derive(Default)]
pub struct RenderStages {
    target_res: (u32, u32),
    interpolation: Option<RenderGroup>,
    kerneled: Option<RenderGroup>,
    min_max: Option<RenderGroup>,
    gamma: Option<RenderGroup>,
    output_staging: Option<RenderGroup>,
}

impl RenderStages {
    pub fn new() -> RenderStages {
        Self::default()
    }

    // Update the resolution of the every stage
    pub fn update_resolution(&mut self, context: &GraphicsContext, dims: (u32, u32)) {
        if self.target_res != dims {
            self.interpolation = Some(RenderGroup::new(
                context,
                dims,
                wgpu::TextureFormat::Rgba32Float,
            ));
            self.kerneled = Some(RenderGroup::new(
                context,
                dims,
                wgpu::TextureFormat::Rgba32Float,
            ));
            self.min_max.get_or_insert_with(|| {
                RenderGroup::new(context, (8, 8), wgpu::TextureFormat::Rgba32Float)
            });
            self.gamma = Some(RenderGroup::new(
                context,
                dims,
                wgpu::TextureFormat::Rgba32Float,
            ));
            self.output_staging = Some(RenderGroup::new(
                context,
                dims,
                wgpu::TextureFormat::Rgba32Float,
            ));
        }
    }

    pub fn interpolation(&self) -> &RenderGroup {
        &self.interpolation.as_ref().unwrap()
    }

    pub fn kerneled(&self) -> &RenderGroup {
        &self.kerneled.as_ref().unwrap()
    }

    pub fn min_max(&self) -> &RenderGroup {
        &self.min_max.as_ref().unwrap()
    }

    pub fn gamma(&self) -> &RenderGroup {
        &self.gamma.as_ref().unwrap()
    }

    pub fn output_staging(&self) -> &RenderGroup {
        &self.output_staging.as_ref().unwrap()
    }
}
