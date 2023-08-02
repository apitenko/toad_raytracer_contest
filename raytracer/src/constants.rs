use crate::math::Vec3;

pub const WINDOW_OUTPUT_ENABLED: bool = true;

pub const RENDER_WIDTH: u32 = 1600;
pub const RENDER_HEIGHT: u32 = 900;
pub const SCALE_FACTOR: u32 = 1;
// pub const RENDER_WIDTH: u32 = 600;
// pub const RENDER_HEIGHT: u32 = 300;
// pub const SCALE_FACTOR: u32 = 2;
// pub const RENDER_WIDTH: u32 = 300;
// pub const RENDER_HEIGHT: u32 = 150;
// pub const SCALE_FACTOR: u32 = 4;


pub const WINDOW_WIDTH: u32 = RENDER_WIDTH * SCALE_FACTOR;
pub const WINDOW_HEIGHT: u32 = RENDER_HEIGHT * SCALE_FACTOR;

pub const RENDER_SIZE: (u32, u32) = (RENDER_WIDTH, RENDER_HEIGHT);

pub const UPDATE_INTERVAL: f32 = 1.0 / 10.0;

pub const MISS_COLOR_VEC3: Vec3 = Vec3::new([144.0 / 256.0, 185.0 / 256.0, 224.0 / 256.0]);
pub const MISS_COLOR: u32 = 224 | (185 << 8) | (144 << 16);


pub const COLOR_SKY_BLUE: Vec3 = Vec3::from_rgb(199, 227, 235);
pub const COLOR_WHITE: Vec3 = Vec3::new([1.0, 1.0, 1.0]);

pub const COLOR_RED: Vec3 = Vec3::new([1.0, 0.0, 0.0]);
pub const COLOR_GREEN: Vec3 = Vec3::new([0.0, 1.0, 0.0]);
pub const COLOR_BLUE: Vec3 = Vec3::new([0.0, 0.0, 1.0]);

pub const COLOR_RED_SCUFF: Vec3 = Vec3::new([0.9, 0.2, 0.2]);
pub const COLOR_GREEN_SCUFF: Vec3 = Vec3::new([0.2, 0.9, 0.2]);
pub const COLOR_BLUE_SCUFF: Vec3 = Vec3::new([0.2, 0.2, 0.9]);

pub const COLOR_CALL_PARAMETERS: Vec3 = Vec3::new([0.5, 0.7, 1.0]);
