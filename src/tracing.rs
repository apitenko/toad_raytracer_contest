pub const MULTISAMPLE_SIZE: usize = 4;

// ? づ｀･ω･)づ I'm too lazy to write a procedural macro

// const MULTISAMPLE_POSITIONS: [(f32, f32); MULTISAMPLE_SIZE] =
//     generate_multisample_positions!(MULTISAMPLE_SIZE);

// ? ... as well as adding conditional compilation of sorts

pub const MULTISAMPLE_OFFSETS: [(f32, f32); MULTISAMPLE_SIZE] =
    [(0.25, 0.25), (0.75, 0.25), (0.75, 0.75), (0.25, 0.75)];
