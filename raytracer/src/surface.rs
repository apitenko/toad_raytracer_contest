use crate::{constants::{MISS_COLOR, RENDER_HEIGHT, WINDOW_HEIGHT}, math::Vec3};

unsafe impl Send for TotallySafeSurfaceWrapper {}
unsafe impl Sync for TotallySafeSurfaceWrapper {}

#[derive(Clone)]
pub struct TotallySafeSurfaceWrapper {
    memory: *mut u32,
    render_size: (u32, u32),
    render_scale: u32,
    surface_size: (u32, u32),
}

impl TotallySafeSurfaceWrapper {
    pub fn new(memory: *mut u32, render_size: (u32, u32), render_scale: u32) -> Self {
        Self {
            memory,
            render_size,
            render_scale,
            surface_size: (render_size.0 * render_scale, render_size.1 * render_scale),
        }
    }

    pub fn write(&mut self, unscaled_position: (u32, u32), data: u32) {
        let render_scale = self.render_scale;

        let scaled_position = (
            unscaled_position.0 * self.render_scale,
            unscaled_position.1 * self.render_scale,
        );

        for i in 0..render_scale {
            for j in 0..render_scale {
                unsafe {
                    let y = WINDOW_HEIGHT - (scaled_position.1 + j) - 1;
                    let x = (scaled_position.0 + i);
                    let index = y * self.surface_size.0 + x;
                    *(self.memory).add(index as usize) = data;
                }
            }
        }
    }
}
