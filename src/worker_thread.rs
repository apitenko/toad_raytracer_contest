use std::thread::JoinHandle;

use crate::{
    constants::{MISS_COLOR, RENDER_HEIGHT, RENDER_WIDTH},
    scene::{workload::Workload, Scene, TotallySafeSceneWrapper},
    surface::TotallySafeSurfaceWrapper,
};

pub struct WorkerThreadHandle {
    thread: JoinHandle<()>,
}

impl WorkerThreadHandle {
    pub fn run(
        mut surface: TotallySafeSurfaceWrapper,
        workload: Workload,
        scene: TotallySafeSceneWrapper,
    ) -> Self {
        let thread = std::thread::spawn(move || {
            for (x, y, index) in workload {
                // Render a pixel
                let red = x % 255;
                let green = y % 255;
                let blue = (x * y) % 255;

                let output = blue | (green << 8) | (red << 16);
                surface.write((x, y), output);
            }
        });
        Self { thread }
    }
}
