use crate::math::Vec3;

pub const WINDOW_OUTPUT_ENABLED: bool = true;

// pub const RENDER_WIDTH: u32 = 1600;
// pub const RENDER_HEIGHT: u32 = 900;
pub const RENDER_WIDTH: u32 = 600;
pub const RENDER_HEIGHT: u32 = 300;

pub const SCALE_FACTOR: u32 = 2;

pub const WINDOW_WIDTH: u32 = RENDER_WIDTH * SCALE_FACTOR;
pub const WINDOW_HEIGHT: u32 = RENDER_HEIGHT * SCALE_FACTOR;

pub const RENDER_SIZE: (u32, u32) = (RENDER_WIDTH, RENDER_HEIGHT);

pub const UPDATE_INTERVAL: f32 = 1.0 / 10.0;

pub const MISS_COLOR_VEC3: Vec3 = Vec3::new([144.0 / 256.0, 185.0 / 256.0, 224.0 / 256.0]);
pub const MISS_COLOR: u32 = 224 | (185 << 8) | (144 << 16);

pub const MAX_BOUNCES: u32 = 50;

pub const COLOR_RED: Vec3 = Vec3::new([1.0, 0.0, 0.0]);
pub const COLOR_GREEN: Vec3 = Vec3::new([0.0, 1.0, 0.0]);