use raytracer_lib::generate_multisample_positions;

// ? づ｀･ω･)づ it's compile time o'clock

generate_multisample_positions!(4);

pub const MULTISAMPLE_OFFSETS: [(f32, f32); 4] = generated_samples();
pub const MULTISAMPLE_SIZE: usize = MULTISAMPLE_OFFSETS.len();

pub const MAX_BOUNCES: i32 = 20;
