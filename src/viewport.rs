use std::default;

use egui::Image;

use crate::context::GraphicsContext;

// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct ViewportDimensions {
//     pub dimensions: [f32; 2],
// }

// impl ViewportDimensions {
//     pub fn from_window(window: &winit::window::Window) -> Self {
//         ViewportDimensions {
//             dimensions: [
//                 window.inner_size().width as f32,
//                 window.inner_size().height as f32,
//             ],
//         }
//     }

//     pub fn from_dim(x: u32, y: u32) -> Self {
//         ViewportDimensions {
//             dimensions: [x as f32, y as f32],
//         }
//     }

//     pub fn bind(&self, context: &GraphicsContext) {
//         context.queue.write_buffer(
//             &context.dim_buffer,
//             0,
//             bytemuck::cast_slice(&[self.dimensions]),
//         )
//     }
// }

#[derive(Copy, Clone, Debug)]
pub struct ImageDisplay {
    pub window_size: [f32; 2],
    pub pos: [f32; 2],
    pub size: f32,
    pub gamma: f32,
    pub scaling_mode: ScalingMode,
    pub cross_correlation: bool,
    pub background_colour: [f32; 3],
}

impl Default for ImageDisplay {
    fn default() -> Self {
        ImageDisplay {
            window_size: [1000., 1000.],
            pos: [0., 0.],
            size: 1.,
            gamma: 1.,
            scaling_mode: ScalingMode::NearestNeighbour,
            cross_correlation: false,
            background_colour: [0., 0., 0.],
        }
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
    pub cross_correlation: u32,
    pub background_colour: [f32; 4],
    pub _pad: [f32; 4],
}

impl ImageDisplay {
    pub fn from_window(window : &winit::window::Window) -> Self {
        ImageDisplay {
            window_size: [
                window.inner_size().width as f32,
                window.inner_size().height as f32,
            ],
            ..Default::default()
        }
    }

    pub fn into_raw(&self) -> RawImageDisplay {
        RawImageDisplay {
            window_size: self.window_size,
            pos: self.pos,
            size: self.size,
            gamma: self.gamma,
            scaling_mode: self.scaling_mode as u32,
            cross_correlation: match self.cross_correlation {
                true => 1,
                false => 0,
            },
            background_colour: [0.0; 4],
            _pad: Default::default(),
        }
    }

    pub fn bind(&mut self, context: &GraphicsContext) {
        context.queue.write_buffer(
            &context.image_display_buffer,
            0,
            bytemuck::bytes_of(&self.into_raw()),
        );
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ScalingMode {
    NearestNeighbour = 0,
    Bilinear = 1,
}
