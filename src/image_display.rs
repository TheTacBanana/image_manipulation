use wgpu::util::DeviceExt;

use crate::context::GraphicsContext;

#[derive(Debug)]
pub struct ImageDisplay {
    pub window_size: [f32; 2],
    pub pos: [f32; 2],
    pub size: f32,
    pub gamma: f32,
    pub scaling_mode: ScalingMode,
    pub cross_correlation: bool,
    pub background_colour: [f32; 4],
    pub layout: wgpu::BindGroupLayout,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl ImageDisplay {
    pub fn from_window(device: &wgpu::Device, window: &winit::window::Window) -> Self {
        let mut raw_image_display = RawImageDisplay::default();
        raw_image_display.window_size = [
            window.inner_size().width as f32,
            window.inner_size().height as f32,
        ];

        let entries = (0..=6)
            .map(|i| wgpu::BindGroupLayoutEntry {
                binding: i,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect::<Vec<wgpu::BindGroupLayoutEntry>>();

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &entries,
            label: Some("image_display_bind_group_layout"),
        });

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("image_display_buf"),
            contents: bytemuck::bytes_of(&raw_image_display),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let entries = (0..=6)
            .map(|i| wgpu::BindGroupEntry {
                binding: i,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            })
            .collect::<Vec<wgpu::BindGroupEntry>>();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &entries,
            label: Some("image_display_bind_group"),
        });

        let RawImageDisplay {
            window_size,
            pos,
            size,
            gamma,
            scaling_mode,
            ..
        } = raw_image_display;

        ImageDisplay {
            window_size,
            pos,
            size,
            gamma,
            scaling_mode: ScalingMode::from_u32(scaling_mode),
            cross_correlation: false,
            layout,
            buffer,
            bind_group,
            background_colour: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub fn into_raw(&self) -> RawImageDisplay {
        RawImageDisplay {
            window_size: self.window_size,
            pos: self.pos,
            size: self.size,
            gamma: self.gamma,
            scaling_mode: self.scaling_mode as u32,
            ..Default::default()
        }
    }

    pub fn reset_default(&mut self) {
        let RawImageDisplay {
            window_size,
            pos,
            size,
            gamma,
            scaling_mode,
            ..
        } = RawImageDisplay::default();

        self.window_size = window_size;
        self.pos = pos;
        self.size = size;
        self.gamma = gamma;
        self.scaling_mode = ScalingMode::from_u32(scaling_mode);
        self.background_colour = [0.0, 0.0, 0.0, 1.0];
    }

    pub fn bind(&self, context: &GraphicsContext) {
        context
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.into_raw()));
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawImageDisplay {
    pub window_size: [f32; 2],
    pub pos: [f32; 2],
    pub size: f32,
    pub gamma: f32,
    pub scaling_mode: u32,
    pub global_min_max: [f32; 2],
    pub _pad: [f32; 5],
}

impl Default for RawImageDisplay {
    fn default() -> Self {
        Self {
            window_size: [1000., 1000.],
            pos: [0., 0.],
            size: 1.,
            gamma: 1.,
            scaling_mode: 0,
            global_min_max: [0.0, 1.0],
            _pad: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ScalingMode {
    NearestNeighbour = 0,
    Bilinear = 1,
}

impl ScalingMode {
    pub fn from_u32(i: u32) -> ScalingMode {
        match i {
            0 => Self::NearestNeighbour,
            1 => Self::Bilinear,
            _ => panic!(),
        }
    }
}
