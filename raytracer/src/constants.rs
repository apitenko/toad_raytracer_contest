use crate::math::Vec3;

pub(crate) const WINDOW_OUTPUT_ENABLED: bool = true;

pub(crate) const UPDATE_INTERVAL: f32 = 1.0 / 10.0;

pub(crate) const DEFAULT_ASPECT_RATIO: f32 = 1.3333333;
pub(crate) const DEFAULT_HEIGHT: u32 = 200;
pub(crate) const DEFAULT_HEIGHT_STRING: &str = const_str::to_str!(DEFAULT_HEIGHT);
pub(crate) const RENDER_SCALE: u32 = 1;

// ? づ｀･ω･)づ it's compile time o'clock

use raytracer_lib::generate_multisample_positions;
generate_multisample_positions!(4);

pub(crate) const MULTISAMPLE_OFFSETS: [(f32, f32); 4] = generated_samples();
pub(crate) const MULTISAMPLE_SIZE: usize = MULTISAMPLE_OFFSETS.len();

pub(crate) const MAX_BOUNCES: i32 = 12;
pub(crate) const MONTE_CARLO_THRESHOLD_BOUNCES: i32 = 1;
// pub const MAX_DEPTH: f32 = 20.0;

// todo: move to skybox
pub(crate) const SKYBOX_LIGHT_INTENSITY: f32 = 0.0;
pub(crate) const SKYBOX_COLOR: Vec3 = COLOR_SKY_BLUE;

pub(crate) const AMBIENT_LIGHT_INTENSITY: f32 = 0.0;
pub(crate) const AMBIENT_LIGHT_COLOR: Vec3 = COLOR_WHITE;

///////////////////////
pub(crate) const MISS_COLOR_VEC3: Vec3 = Vec3::new([144.0 / 256.0, 185.0 / 256.0, 224.0 / 256.0]);
pub(crate) const MISS_COLOR: u32 = 224 | (185 << 8) | (144 << 16);


pub(crate) const COLOR_SKY_BLUE: Vec3 = Vec3::from_rgb(199, 227, 235);
pub(crate) const COLOR_WHITE: Vec3 = Vec3::new([1.0, 1.0, 1.0]);

pub(crate) const COLOR_RED: Vec3 = Vec3::new([1.0, 0.0, 0.0]);
pub(crate) const COLOR_GREEN: Vec3 = Vec3::new([0.0, 1.0, 0.0]);
pub(crate) const COLOR_BLUE: Vec3 = Vec3::new([0.0, 0.0, 1.0]);

pub(crate) const COLOR_RED_SCUFF: Vec3 = Vec3::new([0.9, 0.2, 0.2]);
pub(crate) const COLOR_GREEN_SCUFF: Vec3 = Vec3::new([0.2, 0.9, 0.2]);
pub(crate) const COLOR_BLUE_SCUFF: Vec3 = Vec3::new([0.2, 0.2, 0.9]);

pub(crate) const COLOR_CALL_PARAMETERS: Vec3 = Vec3::new([0.5, 0.7, 1.0]);
