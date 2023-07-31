use raytracer_lib::generate_multisample_positions;

// ? づ｀･ω･)づ it's compile time o'clock

generate_multisample_positions!(40);

pub const MULTISAMPLE_OFFSETS: [(f32, f32); 40] = generated_samples();
pub const MULTISAMPLE_SIZE: usize = MULTISAMPLE_OFFSETS.len();