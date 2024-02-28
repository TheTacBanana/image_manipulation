use wgpu::util::DeviceExt;

use crate::context::GraphicsContext;

/// Store ImageDisplay alongside its layout and buffers
/// Also store change detection
#[derive(Debug)]
pub struct ImageDisplayWithBuffers {
    pub internal: ImageDisplay,
    pub changed: bool,
    pub layout: wgpu::BindGroupLayout,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

/// Data for Image Display
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageDisplay {
    pub window_size: [f32; 2],
    pub pos: [f32; 2],
    pub size: f32,
    pub gamma: f32,
    pub scaling_mode: ScalingMode,
    pub cross_correlation: bool,
    pub background_colour: [f32; 4],
    pub kernel: [f32; 25],
}

/// Raw representation of ImageDisplay for binding to the GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawImageDisplay {
    pub window_size: [f32; 2],
    pub pos: [f32; 2],
    pub size: f32,
    pub gamma: f32,
    pub scaling_mode: u32,
    pub _pad: f32,
}

/// Scaling Mode Enum
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ScalingMode {
    NearestNeighbour = 0,
    Bilinear = 1,
}

impl ImageDisplayWithBuffers {
    /// Create a new ImageDispay and generate buffers for data to be stored in
    pub fn from_window(
        device: &wgpu::Device,
        window: &winit::window::Window,
    ) -> ImageDisplayWithBuffers {
        let mut raw_image_display = RawImageDisplay::default();
        raw_image_display.window_size = [
            window.inner_size().width as f32,
            window.inner_size().height as f32,
        ];

        // Create layout entrys
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

        // Create layout from entries
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &entries,
            label: Some("image_display_bind_group_layout"),
        });

        // Create buffer with intiial contents of default ImageDisplay
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("image_display_buf"),
            contents: bytemuck::bytes_of(&raw_image_display),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group entries
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

        // Create bind group
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

        // Return ImageDisplay
        ImageDisplayWithBuffers {
            changed: true,
            internal: ImageDisplay {
                window_size,
                pos,
                size,
                gamma,
                scaling_mode: ScalingMode::from_u32(scaling_mode),
                cross_correlation: false,
                background_colour: [0.0, 0.0, 0.0, 1.0],
                kernel: *GraphicsContext::LAPLACIAN,
            },
            layout,
            buffer,
            bind_group,
        }
    }

    /// Bind ImageDisplay to the buffer
    pub fn bind(&self, context: &GraphicsContext) {
        context.queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::bytes_of(&self.internal.into_raw()),
        );
    }

    pub fn set_changed(&mut self) {
        self.changed = true;
    }

    pub fn clear_changed(&mut self) {
        self.changed = false;
    }
}

impl ImageDisplay {
    /// Converts an ImageDisplay into RawImageDisplay for binding
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

    /// Reset default values
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
        self.kernel = *GraphicsContext::LAPLACIAN;
        self.cross_correlation = false;
    }
}

impl Default for RawImageDisplay {
    fn default() -> Self {
        Self {
            window_size: [1000., 1000.],
            pos: [0., 0.],
            size: 1.,
            gamma: 1.,
            scaling_mode: 0,
            _pad: Default::default(),
        }
    }
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
