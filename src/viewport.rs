use crate::context::GraphicsContext;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewportDimensions {
    pub dimensions: [f32; 2],
}

impl ViewportDimensions {
    pub fn from_window(window: &winit::window::Window) -> Self {
        ViewportDimensions {
            dimensions: [
                window.inner_size().width as f32,
                window.inner_size().height as f32,
            ],
        }
    }

    pub fn from_dim(x: u32, y: u32) -> Self {
        ViewportDimensions {
            dimensions: [x as f32, y as f32],
        }
    }

    pub fn bind(&self, context: &GraphicsContext) {
        context.queue.write_buffer(
            &context.dim_buffer,
            0,
            bytemuck::cast_slice(&[self.dimensions]),
        )
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ImageDisplay {
    pub pos: [f32; 2],
    pub size: f32,
    pub gamma: f32,
}

impl ImageDisplay {
    pub fn bind(&self, context: &GraphicsContext) {
        context.queue.write_buffer(
            &context.image_display_buffer,
            0,
            bytemuck::bytes_of(self),
        )
    }
}
