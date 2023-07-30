

pub const WINDOW_OUTPUT_ENABLED: bool = true;

// pub const RENDER_WIDTH: u32 = 1600;
// pub const RENDER_HEIGHT: u32 = 900;
pub const RENDER_WIDTH: u32 = 200;
pub const RENDER_HEIGHT: u32 = 110;

pub const SCALE_FACTOR: u32 = 7;

pub const WINDOW_WIDTH: u32 = RENDER_WIDTH * SCALE_FACTOR;
pub const WINDOW_HEIGHT: u32 = RENDER_HEIGHT * SCALE_FACTOR;

pub const RENDER_SIZE: (u32, u32) = (RENDER_WIDTH, RENDER_HEIGHT);

pub const UPDATE_INTERVAL: f32 = 1.0 / 10.0;

pub const MISS_COLOR: u32 = 224 | (185 << 8) | (144 << 16);