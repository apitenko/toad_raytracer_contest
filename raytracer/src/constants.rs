use crate::math::Vec3;

pub const WINDOW_OUTPUT_ENABLED: bool = true;

pub const UPDATE_INTERVAL: f32 = 1.0 / 10.0;

pub const DEFAULT_ASPECT_RATIO: f32 = 1.3333333;
pub const DEFAULT_HEIGHT: u32 = 200;
pub const DEFAULT_HEIGHT_STRING: &str = const_str::to_str!(DEFAULT_HEIGHT);
pub const RENDER_SCALE: u32 = 1;


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
